<template>
    <div class="page">
        <AppSpinner v-if="loading && !item" :rows="4" />

        <AppEmpty
            v-else-if="!item"
            title="没能加载这个条目"
            :hint="error || '检查服务器连接后重试。'"
        >
            <template #actions>
                <el-button type="primary" plain @click="load">重试</el-button>
            </template>
        </AppEmpty>

        <template v-else>
            <DetailHero :title="item.Name" :facts="facts" :overview="item.Overview" bare>
                <template #eyebrow>
                    <button
                        v-if="item.Type !== 'Movie' && item.SeriesId"
                        class="hero-link"
                        type="button"
                        @click="gotoSeries"
                    >
                        {{ item.SeriesName }}
                    </button>
                    <span v-else>电影</span>
                </template>

                <template #actions>
                    <!-- 主操作只有一个实心按钮：继续播放 / 从头播放 -->
                    <el-button
                        v-if="resumePosition > 0 && !item.UserData?.Played"
                        type="primary"
                        size="large"
                        :loading="playing"
                        @click="play(resumePosition)"
                    >
                        <el-icon v-if="!playing"><i-ep-VideoPlay /></el-icon>
                        <span>继续播放 {{ progressPercent }}%</span>
                    </el-button>
                    <el-button v-else type="primary" size="large" :loading="playing" @click="play(0)">
                        <el-icon v-if="!playing"><i-ep-VideoPlay /></el-icon>
                        <span>播放</span>
                    </el-button>

                    <el-button
                        v-if="resumePosition > 0 && !item.UserData?.Played"
                        size="large"
                        :loading="playing"
                        @click="play(0)"
                    >
                        从头播放
                    </el-button>

                    <button
                        class="ghost"
                        type="button"
                        :class="{ 'is-done': item.UserData?.Played }"
                        :disabled="actions.isPlayedPending(item.Id)"
                        @click="actions.togglePlayed(item)"
                    >
                        <el-icon :size="15" :class="{ 'is-loading': actions.isPlayedPending(item.Id) }">
                            <i-ep-CircleCheckFilled v-if="item.UserData?.Played" />
                            <i-ep-CircleCheck v-else />
                        </el-icon>
                        <span>{{ item.UserData?.Played ? '已播放' : '标记已播放' }}</span>
                    </button>

                    <button
                        class="ghost"
                        type="button"
                        :class="{ 'is-on': item.UserData?.IsFavorite }"
                        :disabled="actions.isStarPending(item.Id)"
                        @click="actions.toggleStar(item)"
                    >
                        <el-icon :size="15" :class="{ 'is-loading': actions.isStarPending(item.Id) }">
                            <i-ep-StarFilled v-if="item.UserData?.IsFavorite" />
                            <i-ep-Star v-else />
                        </el-icon>
                        <span>{{ item.UserData?.IsFavorite ? '取消收藏' : '收藏' }}</span>
                    </button>

                    <button class="ghost" type="button" :disabled="playing" @click="play(0, true)">
                        <el-icon :size="15"><i-ep-Download /></el-icon>
                        <span>下载</span>
                    </button>
                </template>

                <template #meta>
                    <ExternalLinks :links="item.ExternalUrls" :provider-ids="item.ProviderIds" />
                </template>

                <template #aside>
                    <div v-if="logo" class="hero-logo">
                        <img v-lazy="logo" :alt="item.Name" />
                    </div>
                </template>
            </DetailHero>

            <!-- 播放配置：版本 / 轨道 / 策略，三行固定节奏 -->
            <AppPanel v-if="versions.length" title="播放配置" caption="开播前先在这里确认版本与轨道">
                <div class="conf">
                    <div class="conf__row">
                        <span class="conf__key">版本</span>
                        <el-select
                            v-model="version"
                            class="conf__version"
                            :disabled="versions.length <= 1"
                            @change="applyVersion"
                        >
                            <el-option
                                v-for="option in versions"
                                :key="option.value"
                                :label="option.name"
                                :value="option.value"
                            >
                                <span class="conf__option">
                                    <span class="t-clip">{{ option.name }}</span>
                                    <span class="tag">{{ option.size }}</span>
                                    <span class="tag">{{ option.bitrate }}</span>
                                    <span class="tag">{{ option.resolution }}</span>
                                </span>
                            </el-option>
                        </el-select>
                    </div>

                    <div class="conf__row">
                        <span class="conf__key">轨道</span>
                        <div class="conf__tracks">
                            <el-select
                                v-model="video"
                                :disabled="videoOptions.length <= 1"
                                placeholder="视频"
                            >
                                <template #prefix><span class="conf__mini">视频</span></template>
                                <el-option
                                    v-for="option in videoOptions"
                                    :key="option.value"
                                    :label="option.label"
                                    :value="option.value"
                                />
                            </el-select>
                            <el-select
                                v-model="audio"
                                :disabled="audioOptions.length <= 1"
                                placeholder="音频"
                            >
                                <template #prefix><span class="conf__mini">音频</span></template>
                                <el-option
                                    v-for="option in audioOptions"
                                    :key="option.value"
                                    :label="option.label"
                                    :value="option.value"
                                />
                            </el-select>
                            <el-select
                                v-model="subtitle"
                                :disabled="subtitleOptions.length <= 1"
                                placeholder="字幕"
                            >
                                <template #prefix><span class="conf__mini">字幕</span></template>
                                <el-option
                                    v-for="option in subtitleOptions"
                                    :key="option.value"
                                    :label="option.label"
                                    :value="option.value"
                                />
                            </el-select>
                        </div>
                    </div>

                    <div class="conf__row">
                        <span class="conf__key">策略</span>
                        <div class="conf__flags">
                            <button
                                class="chip"
                                type="button"
                                :class="{ 'is-on': rememberSelection }"
                                title="开启后，切换下一集时沿用这次的手动选择"
                                @click="rememberSelection = !rememberSelection"
                            >
                                <el-icon :size="13"><i-ep-Pointer /></el-icon>
                                <span>{{ rememberSelection ? '沿用本次选择' : '每集自动选择' }}</span>
                            </button>

                            <button
                                v-if="supportsDirectLink"
                                class="chip"
                                type="button"
                                :class="{ 'is-on': useDirectLink }"
                                title="媒体源在远端时，可让播放器直接取源，绕过本地代理"
                                @click="useDirectLink = !useDirectLink"
                            >
                                <el-icon :size="13"><i-ep-Connection /></el-icon>
                                <span>{{ useDirectLink ? '直链播放' : '走本地代理' }}</span>
                            </button>

                            <span v-if="currentSource" class="conf__facts num">
                                {{ durationText }} · {{ sizeText }} · {{ bitrateText }} · {{ resolutionText }}
                            </span>
                        </div>
                    </div>
                </div>
            </AppPanel>

            <AppSection v-if="item.Chapters?.length" title="章节" :count="item.Chapters.length">
                <ChapterRail :chapters="item.Chapters" @jump="chapter => play(chapter.StartPositionTicks, false, true)" />
            </AppSection>

            <!-- 接下来：同季后续单集 -->
            <AppSection v-if="item.Type !== 'Movie' && item.SeriesId" title="接下来" :count="nextUpTotal">
                <template #actions>
                    <button class="chip" type="button" :class="{ 'is-on': !queryAll }" @click="loadNextUp(1, false)">
                        本季接下来
                    </button>
                    <button class="chip" type="button" :class="{ 'is-on': queryAll }" @click="loadNextUp(1, true)">
                        本季全部
                    </button>
                    <button class="chip" type="button" @click="jumpToNextEpisode">
                        <el-icon :size="13"><i-ep-DArrowRight /></el-icon>
                        <span>下一集</span>
                    </button>
                </template>

                <AppSpinner v-if="nextUpLoading" :rows="3" />

                <div v-else-if="nextUpList.length" class="grid">
                    <ItemPoster
                        v-for="episode in nextUpList"
                        :key="episode.Id"
                        :item="episode"
                        :emby-server-id="serverId"
                        @open="episode => gotoEpisode(episode.Id)"
                    />
                </div>

                <AppEmpty
                    v-else
                    title="这一季已经看到最后"
                    hint="切到「本季全部」可以回看已经看过的单集。"
                />

                <el-pagination
                    v-model:current-page="nextUpPage"
                    :page-size="nextUpPageSize"
                    layout="total, prev, pager, next"
                    :total="nextUpTotal"
                    hide-on-single-page
                    @current-change="(page: number) => loadNextUp(page, queryAll)"
                />
            </AppSection>
        </template>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import AppSection from '../../components/base/AppSection.vue';
import AppPanel from '../../components/base/AppPanel.vue';
import AppEmpty from '../../components/base/AppEmpty.vue';
import AppSpinner from '../../components/base/AppSpinner.vue';
import DetailHero, { type HeroFact } from '../../components/media/DetailHero.vue';
import ChapterRail from '../../components/media/ChapterRail.vue';
import ExternalLinks from '../../components/media/ExternalLinks.vue';
import ItemPoster from '../../components/item/ItemPoster.vue';
import embyApi, { type EpisodeItem } from '../../api/embyApi';
import invokeApi from '../../api/invokeApi';
import { useImage } from '../../store/image';
import { useEventBus } from '../../store/eventBus';
import { useGlobalConfig } from '../../store/db/globalConfig';
import { useItemActions } from '../../composables/useItemActions';
import { usePlaybackTracks, type RestoredSelection } from '../../composables/usePlaybackTracks';
import { formatEpisodeNo, formatYearRange, progressOf } from '../../util/format';
import type { PlaybackNotifyParam } from '../../store/notifyCenter';

/**
 * 播放详情。
 *
 * 一屏之内回答三个问题：
 *  1) 这是什么 —— 标题 / 事实行 / 简介 / 外链
 *  2) 怎么播 —— 主按钮 + 版本 / 轨道 / 策略三行
 *  3) 接下来看什么 —— 章节与同季后续
 *
 * 播放本身交给 Rust 侧的 mpv 集成，这里只负责把参数组织好。
 */
const route = useRoute();
const router = useRouter();
const image = useImage();
const bus = useEventBus();
const serverId = route.params.embyId as string;
const episodeId = route.params.episodeId as string;

const actions = useItemActions(serverId);

const item = ref<EpisodeItem | null>(null);
const loading = ref(false);
const error = ref('');
const playing = ref(false);

const logo = computed(() => image.images[`${serverId}:logo:${episodeId}`] ?? '');
const progressPercent = computed(() => progressOf(item.value?.UserData));
const resumePosition = computed(() => {
    const data = item.value?.UserData;
    return data && !data.Played ? data.PlaybackPositionTicks || 0 : 0;
});

const facts = computed<HeroFact[]>(() => {
    if (!item.value) {
        return [];
    }
    const out: HeroFact[] = [];
    if (item.value.Type === 'Episode') {
        out.push({ value: formatEpisodeNo(item.value.ParentIndexNumber, item.value.IndexNumber), mono: true });
    }
    out.push({ value: formatYearRange(item.value.ProductionYear), mono: true });
    if (item.value.PremiereDate) {
        out.push({ value: item.value.PremiereDate.substring(0, 10), mono: true });
    }
    if (progressPercent.value > 0) {
        out.push({ value: `已看 ${progressPercent.value}%`, mono: true });
    }
    return out;
});

/** 从 URL 读回「记住的选择」，只在带 rememberSelect=true 时生效 */
const rememberSelection = ref(route.query.rememberSelect === 'true');
const useDirectLink = ref(route.query.useDirectLink === 'true');
const policy = ref('high-resolution');

const restored = computed<RestoredSelection | null>(() => {
    if (!rememberSelection.value) {
        return null;
    }
    return {
        version: Number(route.query.versionSelect ?? 1),
        video: Number(route.query.videoSelect ?? -1),
        audio: Number(route.query.audioSelect ?? -1),
        subtitle: Number(route.query.subtitleSelect ?? -1),
    };
});

const {
    versions,
    videoOptions,
    audioOptions,
    subtitleOptions,
    version,
    video,
    audio,
    subtitle,
    current: currentSource,
    durationText,
    sizeText,
    bitrateText,
    resolutionText,
    supportsDirectLink,
    applyItem,
    applyVersion,
} = usePlaybackTracks({
    policy,
    restoredSelection: restored,
    onLog: message => console.debug('[playback]', message),
});

/* —— 加载 —— */
async function load() {
    loading.value = true;
    error.value = '';
    try {
        const result = await embyApi.items(serverId, episodeId);
        item.value = result;
        applyItem(result, true);
        await image.loadLogo(serverId, result);
    } catch (e) {
        error.value = String(e);
        item.value = null;
    } finally {
        loading.value = false;
    }
}

async function loadPolicy() {
    try {
        const value = await useGlobalConfig().getGlobalConfigValue('play_version_auto_select_policy');
        policy.value = value || 'high-resolution';
    } catch {
        policy.value = 'high-resolution';
    }
}

/* —— 播放 —— */
async function play(positionTicks: number, download = false, silent = false) {
    if (!item.value) {
        return;
    }
    if (!silent) {
        playing.value = true;
    }
    try {
        await invokeApi.call_player({
            emby_server_id: serverId,
            series_id: item.value.SeriesId,
            item_id: item.value.Id,
            playback_position_ticks: positionTicks,
            use_direct_link: useDirectLink.value,
            select_policy: rememberSelection.value ? 'manual' : 'auto',
            video_select: video.value,
            audio_select: audio.value,
            subtitle_select: subtitle.value,
            version_select: version.value,
            download,
        });
    } catch (e) {
        ElMessage.error('启动播放失败 ' + e);
    } finally {
        playing.value = false;
    }
}

/* —— 接下来 —— */
const nextUpList = ref<EpisodeItem[]>([]);
const nextUpTotal = ref(0);
const nextUpPage = ref(1);
const nextUpPageSize = ref(12);
const nextUpLoading = ref(false);
const queryAll = ref(false);

async function loadNextUp(page = 1, all = false) {
    if (!item.value?.SeriesId) {
        return;
    }
    queryAll.value = all;
    nextUpPage.value = page;
    nextUpLoading.value = true;
    try {
        const result = await embyApi.episodes(
            serverId,
            item.value.SeriesId,
            item.value.SeasonId,
            (page - 1) * nextUpPageSize.value,
            nextUpPageSize.value,
            all ? undefined : item.value.Id,
        );
        nextUpList.value = result.Items ?? [];
        nextUpTotal.value = result.TotalRecordCount ?? 0;
        await image.warmCovers(serverId, nextUpList.value);
    } catch (e) {
        ElMessage.error('获取后续单集失败 ' + e);
    } finally {
        nextUpLoading.value = false;
    }
}

async function jumpToNextEpisode() {
    if (!item.value?.SeriesId) {
        return;
    }
    try {
        const result = await embyApi.episodes(serverId, item.value.SeriesId, item.value.SeasonId, 1, 1, item.value.Id);
        const next = result.Items?.[0];
        if (!next) {
            ElMessage.warning('已经是这一季的最后一集');
            return;
        }
        gotoEpisode(next.Id);
    } catch (e) {
        ElMessage.error('获取下一集失败 ' + e);
    }
}

/* —— 跳转 —— */
function gotoSeries() {
    if (item.value?.SeriesId) {
        router.push(`/nav/emby/${serverId}/series/${item.value.SeriesId}`);
    }
}

function gotoEpisode(id: string) {
    router.replace({
        path: `/nav/emby/${serverId}/episodes/${id}`,
        query: {
            useDirectLink: String(useDirectLink.value),
            rememberSelect: String(rememberSelection.value),
            videoSelect: video.value,
            audioSelect: audio.value,
            subtitleSelect: subtitle.value,
            versionSelect: version.value,
        },
    });
}

/* —— 播放器联动 —— */
function onPlaybackNotify(payload: PlaybackNotifyParam) {
    if (payload.emby_server_id !== serverId || !item.value) {
        return;
    }
    // 自己播完：静默刷新进度，避免整页骨架闪烁
    if (payload.item_id === item.value.Id && payload.event === 'stop') {
        void refreshQuietly();
        return;
    }
    // 播放器自动播了下一集：页面跟着走
    if (payload.series_id === item.value.SeriesId && payload.item_id !== item.value.Id && payload.event === 'start') {
        gotoEpisode(payload.item_id);
    }
}

async function refreshQuietly() {
    if (!item.value) {
        return;
    }
    try {
        const fresh = await embyApi.items(serverId, item.value.Id);
        // 只更新播放状态，不动用户当前选的轨道
        item.value = { ...fresh, MediaSources: item.value.MediaSources };
    } catch {
        /* 静默失败：进度条晚一点再对也无妨 */
    }
}

onMounted(() => {
    void loadPolicy();
    void load();
    bus.on('playingNotify', onPlaybackNotify);
});

onUnmounted(() => {
    bus.remove('playingNotify', onPlaybackNotify);
});

// 路由 id 变化（换集）时重新载入
router.afterEach(to => {
    if (to.path.includes('/episodes/') && to.params.episodeId !== episodeId) {
        window.setTimeout(() => {
            void load();
            void loadNextUp(1, queryAll.value);
        }, 0);
    }
});
</script>

<style scoped>
.hero-link {
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-faint);
    font-size: var(--fs-md);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease);
}

.hero-link:hover {
    color: var(--lamp);
}

.hero-logo {
    display: flex;
    justify-content: flex-end;
}

.hero-logo img {
    max-width: 20rem;
    max-height: 7rem;
    object-fit: contain;
}

.conf {
    display: flex;
    flex-direction: column;
    gap: 0.875rem;
}

.conf__row {
    display: flex;
    align-items: center;
    gap: 0.875rem;
}

.conf__key {
    flex: none;
    width: 2.25rem;
    color: var(--text-ghost);
    font-size: var(--fs-xs);
}

.conf__version {
    width: min(46rem, 100%);
}

.conf__option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.conf__tracks {
    display: flex;
    flex-wrap: wrap;
    gap: 0.625rem;
}

.conf__tracks :deep(.el-select) {
    width: 14rem;
}

.conf__mini {
    margin-right: 4px;
    color: var(--text-ghost);
    font-size: var(--fs-xs);
}

.conf__flags {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
}

.conf__facts {
    color: var(--text-faint);
    font-size: var(--fs-xs);
}
</style>
