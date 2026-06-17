use super::{albums, artists, lyrics, models::*, songs};
use crate::module::storage::persistent::PersistentStore;

/// 获取歌曲的艺术家列表。
pub fn get_artists_of_song(store: &PersistentStore, song_id: &str) -> Vec<Artist> {
    let song = match songs::get(store, song_id) {
        Some(s) => s,
        None => return vec![],
    };
    song.artist_ids
        .iter()
        .filter_map(|aid| artists::get(store, aid))
        .collect()
}

/// 获取歌曲所属的专辑。
pub fn get_album_of_song(store: &PersistentStore, song_id: &str) -> Option<Album> {
    let song = songs::get(store, song_id)?;
    let album_id = song.album_id?;
    albums::get(store, &album_id)
}

/// 获取歌曲的歌词。
pub fn get_lyric_of_song(store: &PersistentStore, song_id: &str) -> Option<Lyric> {
    lyrics::get_by_song(store, song_id)
}

/// 获取某艺术家的所有歌曲。
pub fn get_songs_by_artist(store: &PersistentStore, artist_id: &str) -> Vec<Song> {
    songs::get_all(store)
        .into_values()
        .filter(|s| s.artist_ids.contains(&artist_id.to_string()))
        .collect()
}

/// 获取某艺术家的所有专辑。
pub fn get_albums_by_artist(store: &PersistentStore, artist_id: &str) -> Vec<Album> {
    albums::get_all(store)
        .into_values()
        .filter(|a| a.artist_id == artist_id)
        .collect()
}

/// 获取专辑中的所有歌曲。
pub fn get_songs_in_album(store: &PersistentStore, album_id: &str) -> Vec<Song> {
    let album = match albums::get(store, album_id) {
        Some(a) => a,
        None => return vec![],
    };
    album
        .song_ids
        .iter()
        .filter_map(|sid| songs::get(store, sid))
        .collect()
}

/// 获取歌曲的所有来源 ID。
pub fn get_source_ids_of_song(store: &PersistentStore, song_id: &str) -> Vec<crate::module::music_source::types::SourceId> {
    songs::get(store, song_id)
        .map(|s| s.source_ids)
        .unwrap_or_default()
}

/// 获取某实体所有来源 ID 的通用方法（按歌曲 ID 查）。
pub fn get_source_ids_of_artist(store: &PersistentStore, artist_id: &str) -> Vec<crate::module::music_source::types::SourceId> {
    artists::get(store, artist_id)
        .map(|a| a.source_ids)
        .unwrap_or_default()
}

/// 获取专辑的所有来源 ID。
pub fn get_source_ids_of_album(store: &PersistentStore, album_id: &str) -> Vec<crate::module::music_source::types::SourceId> {
    albums::get(store, album_id)
        .map(|a| a.source_ids)
        .unwrap_or_default()
}
