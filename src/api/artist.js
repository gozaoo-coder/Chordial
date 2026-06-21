/**
 * 歌手相关 API — 后端 library_* 命令
 *
 * 所有返回值通过 {@link Artist|ArtistSummary} 类包装。
 */
import { transport } from '@/api/transport';
import { Artist, ArtistSummary } from '@/class';

/**
 * 获取歌手完整信息（自动填充 trackIds / albumIds）。
 * @param {string} artistId
 * @returns {Promise<Artist>}
 */
export async function getArtist(artistId) {
  const [data, songs, albums] = await Promise.all([
    transport.command('library_get_artist', { id: artistId }),
    transport.command('library_get_songs_by_artist', { artistId }),
    transport.command('library_get_albums_by_artist', { artistId }),
  ]);
  return new Artist({
    ...data,
    track_ids: (songs || []).map((s) => s.id),
    album_ids: (albums || []).map((a) => a.id),
    track_count: (songs || []).length,
    album_count: (albums || []).length,
  });
}

/**
 * 获取歌手摘要信息。
 * @param {string} artistId
 * @returns {Promise<ArtistSummary>}
 */
export async function getArtistSummary(artistId) {
  const data = await transport.command('library_get_artist', { id: artistId });
  return new ArtistSummary(data);
}

/**
 * 批量获取歌手信息。
 * @param {string[]} artistIds
 * @returns {Promise<Artist[]>}
 */
export async function getArtistsByIds(artistIds) {
  const all = await transport.command('library_get_all_artists');
  return (artistIds || [])
    .map((id) => (all[id] ? new Artist(all[id]) : null))
    .filter(Boolean);
}

/**
 * 获取所有歌手列表。
 * @returns {Promise<ArtistSummary[]>}
 */
export async function getAllArtists() {
  const data = await transport.command('library_get_all_artists');
  return Object.values(data).map((d) => new ArtistSummary(d));
}

/**
 * 获取歌手图片 URL。
 * 注意：后端 Artist 模型不直接包含图片数据，
 * 图片需要通过 SourceId + getAlbumPicture 获取。
 *
 * @param {string} _artistId
 * @returns {Promise<string>} 图片 URL（当前返回空，需通过其他方式获取）
 */
export async function getArtistImageUrl(_artistId) {
  // Artist 模型不含图片二进制数据
  // 歌手图片需通过其作品专辑封面或其他 SourceId 获取
  return '';
}
