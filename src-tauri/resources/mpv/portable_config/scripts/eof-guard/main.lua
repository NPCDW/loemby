--[[
	eof-guard

	Guard against video streams that end prematurely.

	When a stream dies prematurely (e.g. network connection dropped and mpv's
	reconnect attempts failed), the demuxer cache reports EOF at the position
	where the cached data ends, and mpv treats it as the end of the file: it
	stops playback and advances the playlist, even though the file is not over.
	This script checks whether the end of the cached data is actually (close to)
	the end of the file, and if it's not, keeps the file open and, once playback
	reaches the end of the cached data or the user seeks past it, shows a message
	and reloads the file instead of letting the file end.

	Configuration lives in ~~/script-opts/eof-guard.conf
]]

local opt = require('mp.options')

local options = {
	eof_guard = true,
	eof_guard_threshold = 2,
}

opt.read_options(options, 'eof-guard', function() end)

-- Subset of the player state that the guard needs
local state = {
	path = nil,
	duration = nil,
	time = nil,
}

--[[ EOF GUARD ]]

local eof_guard = {
	active = false, -- guard is engaged for the current file
	pos = nil, -- end of the cached data, playback is pinned to it
	path = nil, -- path of the guarded file, reloaded when the guard trips
	reloading = false, -- a reload was already issued for the current guard
	keep_open = nil, -- saved `keep-open` value, restored when guard disengages
	keep_open_pause = nil, -- saved `keep-open-pause` value, restored when guard disengages
}

local function eof_guard_disengage()
	if not eof_guard.active then return end
	eof_guard.active = false
	eof_guard.pos = nil
	eof_guard.path = nil
	eof_guard.reloading = false
	if eof_guard.keep_open then
		mp.set_property('keep-open', eof_guard.keep_open)
		mp.set_property('keep-open-pause', eof_guard.keep_open_pause)
	end
	eof_guard.keep_open = nil
	eof_guard.keep_open_pause = nil
end

local function eof_guard_engage(pos)
	if eof_guard.active then return end
	eof_guard.active = true
	eof_guard.pos = pos
	eof_guard.path = state.path -- saved for reloading, as `state.path` is cleared on end-file
	eof_guard.reloading = false
	eof_guard.keep_open = mp.get_property('keep-open')
	eof_guard.keep_open_pause = mp.get_property('keep-open-pause')
	-- `always` keeps the file open even when it's not the last playlist entry,
	-- which is what stops mpv from advancing to the next file at EOF, while
	-- `keep-open-pause` makes it pause at the last frame instead of moving on.
	mp.set_property('keep-open', 'always')
	mp.set_property('keep-open-pause', 'yes')
end

-- Evaluates whether the current file ended prematurely. `cache_state` can be
-- passed in by callers that already have it to avoid a redundant property read.
local function eof_guard_check(cache_state)
	if not options.eof_guard or eof_guard.active or not state.path then return end
	cache_state = cache_state or mp.get_property_native('demuxer-cache-state')
	-- Only streams that declared EOF are of interest. Anything else is either
	-- still buffering or a file that legitimately ends where its data ends.
	if not cache_state or not cache_state['eof-cached'] then return end
	-- End of the cached data is where mpv is about to stop playback.
	local pos
	for _, range in ipairs(cache_state['seekable-ranges'] or {}) do
		if range['end'] and (not pos or range['end'] > pos) then pos = range['end'] end
	end
	pos = pos or cache_state['cache-end']
	local duration = state.duration
	-- Duration is unknown for some streams (e.g. live ones), in which case we
	-- can't tell a premature EOF from a real one.
	if not (pos and duration and duration > 0) then return end
	if duration - pos > options.eof_guard_threshold then eof_guard_engage(pos) end
end

-- Whether playback ended up beyond the end of the cached data (e.g. the user
-- seeked into an uncached region), which is where the guard trips.
local function eof_guard_past_end()
	return eof_guard.active and state.time and state.time > eof_guard.pos + 0.5
end

-- Reloads the guarded file to recover from the premature EOF. The file is
-- inserted after the current playlist entry, which is then removed, keeping
-- the playlist intact. VODs resume at the position playback was pinned to,
-- while streams are reloaded from their current live position.
local function eof_guard_reload()
	if eof_guard.reloading then return end
	local path = eof_guard.path
	if not path then return end
	eof_guard.reloading = true
	mp.commandv('show-text', '视频流意外中断，正在重新加载…', 10000)
	local duration = mp.get_property_native('duration')
	local time_pos = mp.get_property('time-pos')
	local playlist_pos = mp.get_property_number('playlist-pos')
	if playlist_pos then
		mp.commandv('loadfile', path, 'insert-next')
		mp.commandv('playlist-remove', playlist_pos)
		mp.set_property_number('playlist-pos', playlist_pos)
	else
		mp.commandv('loadfile', path, 'replace')
	end
	local function resume()
		if duration and duration > 0 and time_pos then
			mp.commandv('seek', time_pos, 'absolute')
		end
		mp.set_property('pause', 'no')
		mp.unregister_event(resume)
	end
	mp.register_event('file-loaded', resume)
end

-- Trips the guard, reloading the file instead of letting mpv move on, when
-- playback ends up beyond the end of the cached data.
local function eof_guard_react()
	if eof_guard_past_end() then eof_guard_reload() end
end

-- Cache state is only pushed every once in a while, and can stop being pushed
-- entirely once the stream EOFs, so also check periodically.
local eof_guard_timer
eof_guard_timer = mp.add_timeout(0.5, function()
	eof_guard_check()
	eof_guard_timer:resume()
end)

-- An engaged `keep-open` makes mpv pause at the end of the file *without*
-- firing an `end-file` event, so playback reaching the end of the cached data
-- (which is where mpv considers the file over) has to be detected by watching
-- `eof-reached` instead. The position check makes sure playback actually sits
-- at the end of the cached data, so that a stale `eof-reached` (e.g. right
-- after seeking back into the cache) can't trigger a reload.
mp.observe_property('eof-reached', 'bool', function(_, eof_reached)
	if eof_reached and eof_guard.active and state.time
		and state.time >= eof_guard.pos - 0.5 then
		eof_guard_reload()
	end
end)

--[[ STATE HOOKS ]]

mp.observe_property('path', 'native', function(_, path)
	state.path = path
end)
mp.observe_property('duration', 'native', function(_, duration)
	state.duration = duration
end)
mp.observe_property('playback-time', 'number', function(_, time)
	state.time = time
	-- Reload the file if playback ended up beyond the cached data (e.g. seek)
	eof_guard_react()
end)
mp.observe_property('demuxer-cache-state', 'native', function(_, cache_state)
	eof_guard_check(cache_state)
end)

mp.register_event('file-loaded', function()
	eof_guard_disengage()
end)

mp.register_event('end-file', function(event)
	if event.reason == 'eof' then
		-- Fallback for when an EOF does fire the event (an engaged guard means
		-- the stream died at the end of the cached data before the file was
		-- actually over, so reload it instead of ending it). The usual path is
		-- the `eof-reached` observer, as `keep-open` suppresses this event.
		if eof_guard.active then eof_guard_reload() end
	end
	eof_guard_disengage()
end)
