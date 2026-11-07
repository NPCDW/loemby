#!/usr/bin/env python3
"""Pinned Android libmpv builder. Invoke with python3 -I; no app-build downloads."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import tarfile
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
LOCK_PATH = Path(__file__).with_name('android-mpv.lock.json')
LOCK = json.loads(LOCK_PATH.read_text())
DEFAULT_BUNDLE = ROOT / 'src-tauri/plugins/tauri-plugin-mpv/native'
SYSTEM_LIBS = {'libc.so', 'libm.so', 'libdl.so', 'liblog.so', 'libandroid.so',
               'libz.so', 'libEGL.so', 'libGLESv2.so', 'libGLESv3.so', 'libOpenSLES.so', 'libmediandk.so'}
LINK = '-Wl,-O1,--icf=safe -Wl,-z,max-page-size=16384 -Wl,-z,common-page-size=16384'
ORDER = list(LOCK['sources'])


def fail(message):
    raise RuntimeError(message)


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def run(command, cwd=None, env=None, capture=False):
    print('+ ' + ' '.join(map(str, command)), flush=True)
    return subprocess.run(list(map(str, command)), cwd=cwd or ROOT, env=env,
                          check=True, text=True,
                          stdout=subprocess.PIPE if capture else None).stdout


def download(url, target, expected):
    if target.exists() and sha(target) == expected:
        return
    partial = target.with_suffix(target.suffix + '.partial')
    run(['curl', '--fail', '--location', '--retry', '3', '--connect-timeout', '20',
         '--max-time', '300', url, '-o', partial])
    if sha(partial) != expected:
        fail(f'SHA256 mismatch: {url}')
    partial.replace(target)


def checkout(spec, dest):
    if not (dest / '.git').exists():
        dest.mkdir(parents=True, exist_ok=True)
        run(['git', 'init', dest])
        run(['git', '-C', dest, 'remote', 'add', 'origin', spec['url']])
    current = subprocess.run(['git', '-C', str(dest), 'rev-parse', 'HEAD'],
                             capture_output=True, text=True).stdout.strip()
    if current != spec['commit']:
        run(['git', '-C', dest, 'fetch', '--depth', '1', 'origin', spec['commit']])
        run(['git', '-C', dest, 'checkout', '--detach', spec['commit']])
    dirty = run(['git', '-C', dest, 'status', '--porcelain', '--untracked-files=no'], capture=True)
    if dirty.strip():
        fail(f'Modified source checkout: {dest}')
    # Never follow floating submodule branches or arbitrary URLs from downloaded .gitmodules.
    for path, sub in spec.get('submodules', {}).items():
        gitlink = run(['git', '-C', dest, 'ls-tree', 'HEAD', path], capture=True).split()
        if len(gitlink) < 3 or gitlink[2] != sub['commit']:
            fail(f'Submodule pin mismatch: {path}')
        checkout(sub, dest / path)


def extract(archive, dest):
    # Re-verify exact extracted files before any source build; archives alone are insufficient.
    if dest.exists():
        return
    dest.mkdir()
    with tarfile.open(archive) as t:
        members = t.getmembers()
        roots = {m.name.split('/')[0] for m in members if m.name not in ('.', './')}
        if len(roots) != 1:
            fail(f'Expected one archive root in {archive}')
        for member in members:
            pieces = Path(member.name).parts
            if member.name.startswith('/') or '..' in pieces or member.isdev():
                fail(f'Unsafe archive path: {member.name}')
        t.extractall(dest, filter='data')
        root = dest / next(iter(roots))
        for child in root.iterdir():
            child.rename(dest / child.name)
        root.rmdir()


def find_ndk(arg=None):
    candidates = [arg, os.getenv('ANDROID_NDK_HOME'), os.getenv('ANDROID_NDK_ROOT')]
    for home in [os.getenv('ANDROID_HOME'), os.getenv('ANDROID_SDK_ROOT'), '/opt/android-sdk']:
        if home:
            candidates.append(str(Path(home) / 'ndk' / LOCK['ndk']))
    for item in candidates:
        if item and (Path(item) / 'source.properties').is_file():
            ndk = Path(item).resolve()
            text = (ndk / 'source.properties').read_text()
            if not re.search(r'Pkg.Revision\s*=\s*' + re.escape(LOCK['ndk']) + r'\s*$', text, re.M):
                fail(f'NDK must be {LOCK["ndk"]}, found {ndk}')
            return ndk
    fail(f'Install ndk;{LOCK["ndk"]} with sdkmanager; set ANDROID_NDK_HOME')


def elf(path):
    """Read ELF64 directly; no external parser/tool or host-library allowlist."""
    data = Path(path).read_bytes()
    if data[:6] != b'\x7fELF\x02\x01':
        fail(f'Not little-endian ELF64: {path}')
    header = struct.unpack_from('<HHIQQQIHHHHHH', data, 16)
    if header[0] != 3 or header[1] != 183:
        fail(f'Not AArch64 ET_DYN: {path}')
    phoff, phsize, count = header[4], header[8], header[9]
    loads, dynamic = [], None
    for i in range(count):
        p = struct.unpack_from('<IIQQQQQQ', data, phoff + i * phsize)
        if p[0] == 1:
            if p[7] < 16384 or (p[2] - p[3]) % 16384:
                fail(f'ELF LOAD is not 16KB aligned: {path}')
            loads.append(p)
        if p[0] == 2:
            dynamic = (p[2], p[5])
    if not loads or dynamic is None:
        fail(f'Missing ELF LOAD/DYNAMIC: {path}')
    entries = []
    for pos in range(dynamic[0], dynamic[0] + dynamic[1], 16):
        tag, value = struct.unpack_from('<qQ', data, pos)
        if tag == 0:
            break
        entries.append((tag, value))
    if any(tag == 22 for tag, _ in entries):
        fail(f'Text relocations forbidden: {path}')
    table = next((v for t, v in entries if t == 5), None)
    if table is None:
        fail(f'Missing ELF string table: {path}')
    offset = next((p[2] + table - p[3] for p in loads if p[3] <= table < p[3] + p[5]), None)
    if offset is None:
        fail(f'Invalid string table: {path}')
    def string(index):
        start = offset + index
        return data[start:data.index(b'\0', start)].decode('utf-8')
    needed = [string(v) for t, v in entries if t == 1]
    soname = next((string(v) for t, v in entries if t == 14), None)
    return {'soname': soname, 'needed': needed, 'load_alignment': min(p[7] for p in loads)}


def check(args):
    bundle = Path(args.bundle).resolve()
    manifest_path = bundle / 'manifest.json'
    if not manifest_path.is_file():
        fail(f'Missing native bundle at {bundle}; run python3 -I scripts/android-mpv.py prepare')
    manifest = json.loads(manifest_path.read_text())
    if (manifest.get('schema') != 1 or manifest.get('ndk') != LOCK['ndk']
            or manifest.get('api') != LOCK['api'] or manifest.get('abi') != args.abi
            or manifest.get('lock_sha256') != sha(LOCK_PATH)
            or manifest.get('sources') != LOCK['sources']
            or manifest.get('recipe') != LOCK['recipe']):
        fail('Native manifest does not match pinned lock / ABI / NDK / API')
    actual = {p.relative_to(bundle).as_posix() for p in bundle.rglob('*') if p.is_file() and p != manifest_path}
    if actual != set(manifest['files']):
        fail('Manifest file inventory mismatch (extra or missing bundle files)')
    for rel, entry in manifest['files'].items():
        path = bundle / rel
        if Path(rel).is_absolute() or '..' in Path(rel).parts or path.is_symlink():
            fail(f'Unsafe bundle path: {rel}')
        if sha(path) != entry['sha256']:
            fail(f'Bundle SHA256 mismatch: {rel}')
    for required in ['include/mpv/client.h', 'include/libavcodec/jni.h',
                     f'jniLibs/{args.abi}/libmpv.so', f'jniLibs/{args.abi}/libavcodec.so']:
        if required not in actual:
            fail(f'Missing {required}')
    libraries = sorted((bundle / 'jniLibs' / args.abi).glob('*.so'))
    names = {p.name for p in libraries}
    for library in libraries:
        info = elf(library)
        if info['soname'] != library.name:
            fail(f'SONAME must match packaged filename: {library.name}: {info["soname"]}')
        if manifest['files'][library.relative_to(bundle).as_posix()].get('elf') != info:
            fail(f'ELF manifest mismatch: {library.name}')
        missing = set(info['needed']) - names - SYSTEM_LIBS
        if missing:
            fail(f'DT_NEEDED closure missing for {library.name}: {sorted(missing)}')
        if library.name in SYSTEM_LIBS:
            fail(f'Do not bundle Android platform libraries: {library.name}')
    # libc++_shared is intentionally not a system library; it must be included if needed.
    print(f'Native bundle OK: {args.abi}, {len(libraries)} shared libraries, NDK {LOCK["ndk"]}, API {LOCK["api"]}, 16KB ELF and hashes')


def fetch(args):
    sources = args.sources
    sources.mkdir(parents=True, exist_ok=True)
    checkout(LOCK['recipe'], sources / 'recipe')
    for name, spec in LOCK['sources'].items():
        if 'commit' in spec:
            checkout(spec, sources / name)
        else:
            storage = sources / (name + '-download')
            storage.mkdir(exist_ok=True)
            suffix = ''.join(Path(spec['url']).suffixes)
            archive = storage / ('source' + suffix)
            download(spec['url'], archive, spec['sha256'])
            extract(archive, sources / name)
    print('Sources fetched and checked. Build scripts have not been executed.')


def audit(args):
    recipe = args.sources / 'recipe'
    files = ['buildall.sh', 'include/depinfo.sh', 'include/path.sh'] + [f'scripts/{n}.sh' for n in ORDER]
    for rel in files:
        path = recipe / 'buildscripts' / rel
        upstream = subprocess.check_output(['git', '-C', str(recipe), 'show', f'{LOCK["recipe"]["commit"]}:buildscripts/{rel}'])
        if path.read_bytes() != upstream:
            fail(f'Modified official recipe: {rel}')
        run(['bash', '-n', path])
        print(f'RECIPE {rel} SHA256 {sha(path)}')
        if args.print_recipes:
            print(path.read_text())
    for name, spec in LOCK['sources'].items():
        source = args.sources / name
        if not source.is_dir():
            fail(f'Source missing: {source}; run fetch first')
        if 'commit' in spec:
            current = run(['git', '-C', source, 'rev-parse', 'HEAD'], capture=True).strip()
            if current != spec['commit']:
                fail(f'Source revision mismatch: {name}')
            pending = [(source, spec)]
            while pending:
                tree, pin = pending.pop()
                dirty = run(['git', '-C', tree, 'status', '--porcelain', '--untracked-files=no'], capture=True)
                if dirty.strip():
                    fail(f'Modified tracked source files: {tree}')
                for rel, sub in pin.get('submodules', {}).items():
                    subdir = tree / rel
                    revision = run(['git', '-C', subdir, 'rev-parse', 'HEAD'], capture=True).strip()
                    if revision != sub['commit']:
                        fail(f'Submodule source revision mismatch: {name}/{rel}')
                    pending.append((subdir, sub))
        else:
            archive = next((args.sources / (name + '-download')).glob('source.*'))
            if sha(archive) != spec['sha256']:
                fail(f'Archive mismatch: {name}')
            with tarfile.open(archive) as tar:
                for member in tar.getmembers():
                    if member.isfile():
                        relative = Path(*Path(member.name).parts[1:])
                        path = source / relative
                        expected = hashlib.sha256(tar.extractfile(member).read()).hexdigest()
                        if not path.is_file() or path.is_symlink() or sha(path) != expected:
                            fail(f'Modified extracted source file: {name}/{relative}')
    print('Official recipes audited; local implementation uses absolute source paths, API24 and NDK r27c instead of r30.')


def crossfile(prefix, toolchain):
    content = f"""[built-in options]
    buildtype = 'release'
    default_library = 'static'
    wrap_mode = 'nodownload'
    prefix = '/usr/local'
    c_args = ['-fPIC']
    cpp_args = ['-fPIC']
    c_link_args = ['-Wl,-z,max-page-size=16384', '-Wl,-z,common-page-size=16384']
    cpp_link_args = ['-Wl,-z,max-page-size=16384', '-Wl,-z,common-page-size=16384']
    [binaries]
    c = '{toolchain}/bin/aarch64-linux-android24-clang'
    cpp = '{toolchain}/bin/aarch64-linux-android24-clang++'
    ar = '{toolchain}/bin/llvm-ar'
    nm = '{toolchain}/bin/llvm-nm'
    strip = '{toolchain}/bin/llvm-strip'
    pkg-config = 'pkg-config'
    [host_machine]
    system = 'android'
    cpu_family = 'aarch64'
    cpu = 'aarch64'
    endian = 'little'
    """
    path = prefix / 'crossfile.txt'
    path.write_text(content)
    return path


def build(args):
    audit(args)
    if not args.accept_reviewed_recipes:
        fail('Inspect audit --print-recipes and source build files, then explicitly pass --accept-reviewed-recipes')
    ndk = find_ndk(args.ndk)
    toolchain = next((ndk / 'toolchains/llvm/prebuilt').glob('linux-*'))
    work = args.work
    if work == args.sources or args.sources in work.parents or work in args.sources.parents:
        fail('--work and --sources must be separate directories, not nested')
    work.mkdir(parents=True, exist_ok=True)
    # Build tools and interpreter cwd are outside every downloaded source/archive tree.
    prefix = work / 'prefix'
    prefix.mkdir(exist_ok=True)
    for link in ['usr', 'local']:
        if not (prefix / link).exists():
            (prefix / link).symlink_to('.')
    cross = crossfile(prefix, toolchain)
    env = os.environ.copy()
    env.pop('PYTHONPATH', None)
    env.pop('PYTHONHOME', None)
    env.update(CC=str(toolchain / 'bin/aarch64-linux-android24-clang'),
               CXX=str(toolchain / 'bin/aarch64-linux-android24-clang++'),
               AR='llvm-ar', RANLIB='llvm-ranlib', NM='llvm-nm',
               CFLAGS='-fPIC', CXXFLAGS='-fPIC', LDFLAGS=LINK,
               PKG_CONFIG_SYSROOT_DIR=str(prefix), PKG_CONFIG_LIBDIR=str(prefix / 'lib/pkgconfig'),
               PKG_CONFIG_PATH='', PATH=f'{toolchain}/bin:{env["PATH"]}')
    env.update(ANDROID_NDK_ROOT=str(ndk), ANDROID_NDK_HOME=str(ndk))
    for tool in ['make', 'ninja', 'pkg-config', 'autoreconf', 'gperf', 'nasm']:
        if not shutil.which(tool, path=env['PATH']):
            fail(f'Missing build tool {tool}; see docs/android-mpv.md')
    meson = shutil.which('meson', path=env['PATH'])
    if not meson or run([meson, '--version'], capture=True).strip() != LOCK['meson']['version']:
        fail(f'Install pinned Meson {LOCK["meson"]["version"]}; see docs/android-mpv.md')
    state = {'lock_sha256': sha(LOCK_PATH), 'ndk': str(ndk), 'api': LOCK['api'], 'link_flags': LINK}
    state_path = work / 'build-state.json'
    if state_path.exists() and json.loads(state_path.read_text()) != state:
        fail('Work directory belongs to a different lock/toolchain; use a fresh --work directory')
    state_path.write_text(json.dumps(state, indent=2) + '\n')
    meson_options = {
        'dav1d': ['-Denable_tests=false', '-Db_lto=true', '-Dstack_alignment=16'],
        'libxml2': ['-Dminimum=true', '-Dpush=enabled', '-Dreader=enabled', '-Dsax1=enabled', '-Diso8859x=enabled', '-Dpattern=enabled'],
        'freetype2': ['-Dharfbuzz=disabled', '-Dbrotli=disabled', '-Dpng=disabled', '-Dbzip2=disabled'],
        'fontconfig': ['-Dtests=disabled', '-Ddoc=disabled', '-Dtools=disabled', '-Dnls=disabled', '-Dxml-backend=libxml2'],
        'fribidi': ['-Dtests=false', '-Ddocs=false'],
        'harfbuzz': ['-Dtests=disabled', '-Ddocs=disabled', '-Draster=disabled', '-Dvector=disabled', '-Dgpu=disabled', '-Dsubset=disabled'],
        'libplacebo': ['-Dvulkan=disabled', '-Ddemos=false', '-Dshaderc=disabled', '-Dglslang=disabled'],
        'mpv': ['--default-library=shared', '-Diconv=disabled', '-Dlua=enabled', '-Dlibcurl=enabled', '-Dlibmpv=true', '-Dcplayer=false', '-Dmanpage-build=disabled', '-Dvulkan=disabled', '-Dopensles=enabled', '-Degl-android=enabled']}
    def execute(cmd, cwd=None, extra=None):
        record = {'command': list(map(str, cmd)), 'cwd': str(cwd or work),
                  'environment': {key: (env | (extra or {})).get(key) for key in
                      ['CC', 'CXX', 'AR', 'RANLIB', 'CFLAGS', 'CXXFLAGS', 'LDFLAGS',
                       'PKG_CONFIG_SYSROOT_DIR', 'PKG_CONFIG_LIBDIR', 'DESTDIR']}}
        with (work / 'build-commands.jsonl').open('a') as log:
            log.write(json.dumps(record, sort_keys=True) + '\n')
        run(cmd, cwd=cwd or work, env=env | (extra or {}))
    for name in ORDER:
        source = args.sources / name
        directory = work / ('build-' + name)
        stamp = work / (name + '.complete')
        if stamp.exists():
            continue
        print(f'BUILD {name}', flush=True)
        if name in meson_options:
            command = [meson, 'setup', directory, source, '--cross-file', cross] + meson_options[name]
            if (directory / 'meson-private/coredata.dat').exists():
                command.append('--reconfigure')
            execute(command)
            execute(['ninja', '-C', directory, f'-j{args.jobs}'])
            execute(['ninja', '-C', directory, 'install'], extra={'DESTDIR': str(prefix)})
            if name == 'libplacebo':
                pc = prefix / 'lib/pkgconfig/libplacebo.pc'
                pc.write_text(re.sub(r'^(Libs:.*)$', r'\1 -lc++', pc.read_text(), flags=re.M))
        elif name == 'mbedtls':
            # mbedTLS builds in-tree; keep the verified extracted source immutable.
            if not directory.exists():
                shutil.copytree(source, directory, ignore=shutil.ignore_patterns('*.o', '*.a', '*.so', '__pycache__'))
            # config.py intentionally imports its reviewed sibling framework modules.
            execute([sys.executable, directory / 'scripts/config.py', 'unset', 'MBEDTLS_AESNI_C'])
            execute([sys.executable, directory / 'scripts/config.py', 'set', 'MBEDTLS_PLATFORM_DEV_RANDOM', '"/dev/urandom"'])
            execute(['make', '-C', directory, f'-j{args.jobs}', 'no_test'])
            execute(['make', '-C', directory, f'DESTDIR={prefix}', 'install'])
        elif name == 'lua':
            execute(['make', '-C', source, f'CC={env["CC"]}', 'AR=llvm-ar rc', 'RANLIB=llvm-ranlib', 'MYCFLAGS=-fPIC -Dgetlocaledecpoint\\(\\)=\\(46\\) -Dlua_fseek', 'PLAT=linux', 'LUA_T=', 'LUAC_T=', f'-j{args.jobs}'])
            execute(['make', '-C', source, f'INSTALL_TOP={prefix}', 'TO_BIN=/dev/null', 'install'])
            (prefix / 'lib/pkgconfig/lua.pc').write_text(f'prefix={prefix}\nlibdir=${{prefix}}/lib\nincludedir=${{prefix}}/include\nName: Lua\nDescription: Lua\nVersion: 5.2.4\nLibs: -L${{libdir}} -llua\nCflags: -I${{includedir}}\n')
        else:
            directory.mkdir(exist_ok=True)
            if name == 'libass' and not (source / 'configure').exists():
                execute(['autoreconf', '-fi', source])
            if name == 'ffmpeg':
                options = ['--target-os=android', '--enable-cross-compile', '--cross-prefix=aarch64-linux-android-', f'--cc={env["CC"]}', '--pkg-config=pkg-config', '--nm=llvm-nm', '--arch=aarch64', '--cpu=armv8-a', f'--extra-cflags=-I{prefix}/include -fPIC', f'--extra-ldflags=-L{prefix}/lib {LINK}', '--enable-jni', '--enable-mediacodec', '--enable-mbedtls', '--enable-libdav1d', '--enable-libxml2', '--disable-vulkan', '--disable-static', '--enable-shared', '--enable-gpl', '--enable-version3', '--disable-stripping', '--disable-doc', '--disable-programs', '--disable-muxers', '--disable-encoders', '--disable-devices', '--enable-encoder=mjpeg,png', '--enable-muxer=mov,matroska,mpegts']
            elif name == 'curl':
                options = ['--host=aarch64-linux-android', f'--with-mbedtls={prefix}', '--without-libpsl', '--disable-shared', '--enable-static', '--disable-debug'] + ['--disable-' + f for f in 'manual docs ares unix-sockets tls-srp doh rtsp dict telnet tftp pop3 imap smb smtp gopher mqtt ntlm'.split()]
            else:
                options = ['--host=aarch64-linux-android', '--with-pic', '--enable-static', '--disable-shared']
                if name == 'libass':
                    options += ['--enable-asm', '--enable-libunibreak', '--enable-fontconfig']
            execute([source / 'configure'] + options, cwd=directory)
            execute(['make', '-C', directory, f'-j{args.jobs}'])
            execute(['make', '-C', directory, f'DESTDIR={prefix}', 'install'])
        stamp.write_text(sha(LOCK_PATH) + '\n')
    package(args, ndk, prefix, toolchain)


def package(args, ndk, prefix, toolchain):
    destination = Path(args.bundle).resolve()
    destination.parent.mkdir(parents=True, exist_ok=True)
    staged = Path(tempfile.mkdtemp(prefix='android-mpv-staging-', dir=destination.parent))
    try:
        for header in ['mpv/client.h', 'libavcodec/jni.h']:
            target = staged / 'include' / header
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(prefix / 'include' / header, target)
        libs = staged / 'jniLibs' / LOCK['abi']
        libs.mkdir(parents=True)
        for source in sorted((prefix / 'lib').glob('*.so')):
            shutil.copyfile(source.resolve(), libs / source.name)
        runtime = toolchain / 'sysroot/usr/lib/aarch64-linux-android/libc++_shared.so'
        needed = {n for p in libs.glob('*.so') for n in elf(p)['needed']}
        if 'libc++_shared.so' in needed:
            shutil.copyfile(runtime, libs / runtime.name)
        # Include all upstream notices, including nested third-party code and submodules.
        licenses = []
        for name, spec in LOCK['sources'].items():
            paths = []
            for p in (args.sources / name).rglob('*'):
                if '.git' in p.parts or not p.is_file() or p.is_symlink():
                    continue
                if re.match(r'^(COPYING|LICENSE|LICENCE|COPYRIGHT|NOTICE|FTL\.TXT)', p.name, re.I) or (name == 'lua' and p.relative_to(args.sources / name).as_posix() in ['doc/readme.html', 'src/lua.h']):
                    rel = p.relative_to(args.sources / name)
                    target = staged / 'licenses' / name / rel
                    target.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(p, target)
                    paths.append(target.relative_to(staged).as_posix())
            if not paths:
                fail(f'No license notices found: {name}')
            licenses.append({'source': name, 'license': spec['license'], 'files': paths})
        # r27c ships libc++ notices in the aggregate toolchain notice, not sources/cxx-stl.
        for notice in ['NOTICE', 'NOTICE.toolchain']:
            runtime_notice = ndk / notice
            if not runtime_notice.is_file():
                fail(f'NDK notice missing: {notice}')
            shutil.copyfile(runtime_notice, staged / 'licenses' / ('ndk-' + notice))
        actual_versions = {}
        for name in ORDER:
            info = args.work / ('build-' + name) / 'meson-info/intro-projectinfo.json'
            if info.exists():
                actual_versions[name] = json.loads(info.read_text())['version']
            elif name == 'libass':
                pc = prefix / 'lib/pkgconfig/libass.pc'
                actual_versions[name] = re.search(r'^Version:\s*(.+)$', pc.read_text(), re.M).group(1)
            else:
                actual_versions[name] = LOCK['sources'][name]['version']
        tools = {}
        for tool, command in {'meson': ['meson', '--version'], 'ninja': ['ninja', '--version'],
                              'autoconf': ['autoconf', '--version'], 'automake': ['automake', '--version'],
                              'libtool': ['libtoolize', '--version'], 'make': ['make', '--version'],
                              'clang': [toolchain / 'bin/clang', '--version']}.items():
            tools[tool] = run(command, capture=True).splitlines()[0]
        manifest = {'schema': 1, 'abi': LOCK['abi'], 'ndk': LOCK['ndk'], 'api': LOCK['api'],
                    'actual_versions': actual_versions, 'build_tools': tools,
                    'builder_sha256': sha(Path(__file__).resolve()),
                    'recipe': LOCK['recipe'], 'sources': LOCK['sources'], 'licenses': licenses,
                    'lock_sha256': sha(LOCK_PATH), 'link_flags': LINK,
                    'build_state': json.loads((args.work / 'build-state.json').read_text()),
                    'build_commands': [json.loads(line) for line in (args.work / 'build-commands.jsonl').read_text().splitlines()],
                    'recipes': {p.relative_to(args.sources / 'recipe').as_posix(): sha(p) for p in (args.sources / 'recipe/buildscripts').rglob('*.sh')},
                    'files': {}}
        for p in sorted(staged.rglob('*')):
            if p.is_file():
                entry = {'sha256': sha(p), 'size': p.stat().st_size}
                if p.suffix == '.so':
                    entry['elf'] = elf(p)
                manifest['files'][p.relative_to(staged).as_posix()] = entry
        (staged / 'manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
        check(argparse.Namespace(bundle=staged, abi=LOCK['abi']))
        if destination.exists():
            if any(destination.iterdir()):
                fail(f'Output exists: {destination}; move it aside explicitly before preparing again')
            destination.rmdir()
        staged.rename(destination)
        print(f'Native bundle prepared at {destination}')
    except BaseException:
        shutil.rmtree(staged)
        raise


def check_apk(args):
    apk = Path(args.apk).resolve()
    with tempfile.TemporaryDirectory(prefix='loemby-apk-check-') as tmp:
        with zipfile.ZipFile(apk) as z:
            names = set()
            infos = []
            for item in z.infolist():
                if item.filename.startswith('lib/') and item.filename.endswith('.so'):
                    if not item.filename.startswith('lib/arm64-v8a/'):
                        fail(f'Unexpected APK ABI: {item.filename}')
                    if item.filename in names:
                        fail(f'Duplicate APK native entry: {item.filename}')
                    names.add(item.filename)
                    target = Path(tmp) / Path(item.filename).name
                    target.write_bytes(z.read(item))
                    infos.append((target.name, elf(target)))
            if not infos:
                fail('APK has no arm64 native libraries')
            basenames = {name for name, _ in infos}
            for name, info in infos:
                missing = set(info['needed']) - basenames - SYSTEM_LIBS
                if missing:
                    fail(f'APK native dependency missing: {name}: {missing}')
    ndk = find_ndk(args.ndk)
    sdk = ndk.parents[1]
    zipalign = sdk / 'build-tools/36.0.0/zipalign'
    if not zipalign.exists():
        fail('Install build-tools;36.0.0 for APK 16KB zip alignment check')
    run([zipalign, '-c', '-P', '16', '-v', '4', apk])
    print('APK native ELF closure / ABI / 16KB ELF and zip alignment OK')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    for command in ['fetch', 'audit', 'build', 'prepare']:
        p = sub.add_parser(command)
        p.add_argument('--sources', type=lambda v: Path(v).resolve(), default=Path('/tmp/loemby-android-mpv-sources'))
        p.add_argument('--work', type=lambda v: Path(v).resolve(), default=Path('/tmp/loemby-android-mpv-build'))
        p.add_argument('--bundle', default=os.getenv('LOEMBY_MPV_NATIVE_DIR', str(DEFAULT_BUNDLE)))
        p.add_argument('--ndk')
        p.add_argument('--jobs', type=int, default=min(os.cpu_count() or 4, 8))
        p.add_argument('--print-recipes', action='store_true')
        p.add_argument('--accept-reviewed-recipes', action='store_true')
    p = sub.add_parser('check')
    p.add_argument('--bundle', default=os.getenv('LOEMBY_MPV_NATIVE_DIR', str(DEFAULT_BUNDLE)))
    p.add_argument('--abi', choices=['arm64-v8a'], default='arm64-v8a')
    p.add_argument('--ndk', help='Accepted for Gradle integration; ELF parser does not require NDK')
    p = sub.add_parser('check-apk')
    p.add_argument('--apk', required=True)
    p.add_argument('--ndk')
    args = parser.parse_args()
    try:
        if args.command == 'fetch':
            fetch(args)
        elif args.command == 'audit':
            audit(args)
        elif args.command in ['build', 'prepare']:
            if args.command == 'prepare':
                fetch(args)
            build(args)
        elif args.command == 'check':
            check(args)
        else:
            check_apk(args)
    except (RuntimeError, OSError, ValueError, KeyError, struct.error, subprocess.CalledProcessError) as exc:
        print(f'android-mpv: {exc}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
