<template>
    <div class="page">
        <AppSpinner v-if="seriesLoading && !series" :rows="4" />

        <AppEmpty
            v-else-if="!series"
            title="没能加载这部剧"
            :hint="error || '检查服务器连接后重试。'"
        >
            <template #actions>
                <el-button type="primary" plain @click="loadSeries">重试</el-button>
            </template>
        </AppEmpty>

        <template v-else>
            <DetailHero
                :title="series.Name"
                :poster="poster"
                :facts="facts"
                :overview="series.Overview"
                bare
            >
                <template #actions>
                    <button
                        class="ghost"
                        type="button"
                        :class="{ 'is-done': series.UserData?.Played }"
                        :disabled="actions.isPlayedPending(series.Id)"
                        @click="actions.togglePlayed(series)"
                    >
                        <el-icon :size="15" :class="{ 'is-loading': actions.isPlayedPending(series.Id) }">
                            <i-ep-CircleCheckFilled v-if="series.UserData?.Played" />
                            <i-ep-CircleCheck v-else />
                        </el-icon>
                        <span>{{ series.UserData?.Played ? '已播放' : '标记已播放' }}</span>
                    </button>

                    <button
                        class="ghost"
                        type="button"
                        :class="{ 'is-on': series.UserData?.IsFavorite }"
                        :disabled="actions.isStarPending(series.Id)"
                        @click="actions.toggleStar(series)"
                    >
                        <el-icon :size="15" :class="{ 'is-loading': actions.isStarPending(series.Id) }">
                            <i-ep-StarFilled v-if="series.UserData?.IsFavorite" />
                            <i-ep-Star v-else />
                        </el-icon>
                        <span>{{ series.UserData?.IsFavorite ? '取消收藏' : '收藏' }}</span>
                    </button>
                </template>

                <template #meta>
                    <ExternalLinks :links="series.ExternalUrls" :provider-ids="series.ProviderIds" />
                </template>
            </DetailHero>

            <!-- 季条：点击即在下方展开该季的单集 -->
            <AppSection v-if="seasons.length" title="季" :count="seasons.length">
                <div class="shelf">
                    <button
                        v-for="season in seasons"
                        :key="season.Id"
                        class="season clickable"
                        :class="{ 'is-active': activeSeason?.Id === season.Id }"
                        type="button"
                        :title="season.Name"
                        @click="selectSeason(season)"
                    >
                        <span class="cover cover--poster season__art">
                            <img v-if="coverOf(season.Id)" v-lazy="coverOf(season.Id)" :alt="season.Name" />
                            <span v-else class="season__blank" />
                            <span v-if="season.UserData?.UnplayedItemCount" class="corner">
                                {{ season.UserData.UnplayedItemCount }}
                            </span>
                        </span>
                        <span class="season__name t-clip">{{ season.Name }}</span>
                        <span class="season__meta num">
                            {{ season.ProductionYear || '' }}
                        </span>
                    </button>
                </div>
            </AppSection>

            <!-- 单集列表：行式，一屏看完一季 -->
            <AppSection
                :title="activeSeason ? `单集 · ${activeSeason.Name}` : '单集'"
                :count="episodeTotal"
            >
                <template #actions>
                    <button
                        v-if="activeSeason"
                        class="ghost ghost--bare"
                        type="button"
                        @click="selectSeason(activeSeason)"
                    >
                        刷新
                    </button>
                </template>

                <AppSpinner v-if="episodesLoading" :rows="6" />

                <div v-else-if="episodes.length" class="episodes">
                    <EpisodeRow
                        v-for="episode in episodes"
                        :key="episode.Id"
                        :item="episode"
                        :star-pending="actions.isStarPending(episode.Id)"
                        :played-pending="actions.isPlayedPending(episode.Id)"
                        @open="item => gotoEpisode(item.Id)"
                        @star="actions.toggleStar(episode)"
                        @played="actions.togglePlayed(episode)"
                    />
                </div>

                <AppEmpty
                    v-else
                    title="这一季还没有单集"
                    hint="换一季看看，或等服务器完成扫描。"
                />

                <el-pagination
                    v-model:current-page="episodePage"
                    :page-size="episodePageSize"
                    layout="total, prev, pager, next"
                    :total="episodeTotal"
                    hide-on-single-page
                    @current-change="loadEpisodes"
                />
            </AppSection>
        </template>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import AppSection from '../../components/base/AppSection.vue';
import AppEmpty from '../../components/base/AppEmpty.vue';
import AppSpinner from '../../components/base/AppSpinner.vue';
import DetailHero, { type HeroFact } from '../../components/media/DetailHero.vue';
import ExternalLinks from '../../components/media/ExternalLinks.vue';
import EpisodeRow from '../../components/item/EpisodeRow.vue';
import embyApi, { type EpisodeItem, type SeasonItem, type SeriesItem } from '../../api/embyApi';
import { useImage } from '../../store/image';
import { useItemActions } from '../../composables/useItemActions';
import { formatYearRange } from '../../util/format';

/**
 * 剧集详情。
 *
 * 结构与「播放详情」保持一致：主视觉在左，事实行在标题下，
 * 季用横向条，单集用行式列表——点击季条就在下方就地展开，不再弹二级弹窗。
 */
const route = useRoute();
const router = useRouter();
const image = useImage();
const serverId = route.params.embyId as string;
const seriesId = route.params.serieId as string;

const actions = useItemActions(serverId);

/* —— 剧集本身 —— */
const series = ref<SeriesItem | null>(null);
const seriesLoading = ref(false);
const error = ref('');

const poster = computed(() => image.images[`${serverId}:cover:${seriesId}`] ?? '');

const facts = computed<HeroFact[]>(() => {
    if (!series.value) {
        return [];
    }
    const out: HeroFact[] = [];
    out.push({ value: formatYearRange(series.value.ProductionYear, series.value.EndDate), mono: true });
    if (seasons.value.length) {
        out.push({ value: `${seasons.value.length} 季`, mono: true });
    }
    if (series.value.UserData?.UnplayedItemCount) {
        out.push({ value: `未看 ${series.value.UserData.UnplayedItemCount} 集`, mono: true });
    }
    if (series.value.OfficialRating) {
        out.push({ value: series.value.OfficialRating, mono: false });
    }
    for (const genre of (series.value.Genres || []).slice(0, 4)) {
        out.push({ value: genre, mono: false });
    }
    return out;
});

async function loadSeries() {
    seriesLoading.value = true;
    error.value = '';
    try {
        const result = await embyApi.items(serverId, seriesId);
        series.value = embyApi.asSeries(result);
        await image.loadCover(serverId, result);
    } catch (e) {
        error.value = String(e);
        series.value = null;
    } finally {
        seriesLoading.value = false;
    }
}

/* —— 季 —— */
const seasons = ref<SeasonItem[]>([]);

function coverOf(itemId: string): string {
    return image.images[`${serverId}:cover:${itemId}`] ?? '';
}

async function loadSeasons() {
    try {
        const result = await embyApi.seasons(serverId, seriesId);
        seasons.value = result.Items ?? [];
        await image.warmCovers(serverId, seasons.value);
        if (seasons.value.length) {
            await selectSeason(seasons.value[0]);
        }
    } catch (e) {
        ElMessage.error('获取季列表失败 ' + e);
    }
}

/* —— 单集 —— */
const activeSeason = ref<SeasonItem | null>(null);
const episodes = ref<EpisodeItem[]>([]);
const episodeTotal = ref(0);
const episodePage = ref(1);
const episodePageSize = ref(30);
const episodesLoading = ref(false);

async function selectSeason(season: SeasonItem, targetPage = 1) {
    activeSeason.value = season;
    episodePage.value = targetPage;
    episodesLoading.value = true;
    try {
        const result = await embyApi.episodes(
            serverId,
            seriesId,
            season.Id,
            (targetPage - 1) * episodePageSize.value,
            episodePageSize.value,
        );
        episodes.value = result.Items ?? [];
        episodeTotal.value = result.TotalRecordCount ?? 0;
    } catch (e) {
        ElMessage.error('获取单集失败 ' + e);
    } finally {
        episodesLoading.value = false;
    }
}

function loadEpisodes(targetPage: number) {
    if (activeSeason.value) {
        void selectSeason(activeSeason.value, targetPage);
    }
}

function gotoEpisode(episodeId: string) {
    router.push(`/nav/emby/${serverId}/episodes/${episodeId}`);
}

onMounted(() => {
    void loadSeries();
    void loadSeasons();
});
</script>

<style scoped>
.season {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    width: 7.25rem;
    padding: 0;
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
}

.season__art {
    display: block;
    width: 100%;
}

.season__art img {
    width: 100%;
    height: 100%;
}

.season__blank {
    display: block;
    width: 100%;
    height: 100%;
    background: var(--surface-hover);
}

.season.is-active .season__art {
    outline-color: var(--lamp);
}

.season__name {
    color: var(--text-dim);
    font-size: var(--fs-sm);
}

.season:hover .season__name,
.season.is-active .season__name {
    color: var(--text);
}

.season__meta {
    color: var(--text-ghost);
    font-size: var(--fs-xs);
}

.episodes {
    display: flex;
    flex-direction: column;
}
</style>
