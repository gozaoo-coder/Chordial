/**
 * SourceId — 来源标识
 *
 * 对应后端 `music_source::types::SourceId`。
 */
export class SourceId {
  /** @param {Partial<SourceId>} data */
  constructor(data = {}) {
    /** 来源名称，如 "my_local", "netease", "spotify" */
    this.sourceName = data.source_name ?? data.sourceName ?? '';
    /** 来源类型: "Local" | {"Web": "name"} */
    this.sourceType = data.source_type ?? data.sourceType ?? 'Local';
    /** 实体类型: "Song" | "Artist" | "Album" | "Lyric" */
    this.entityType = data.entity_type ?? data.entityType ?? 'Song';
    /** 该来源内部的实体 ID */
    this.entityId = data.entity_id ?? data.entityId ?? '';
  }

  /** 快捷判断是否属于本地来源 */
  isLocal() {
    return this.sourceType === 'Local';
  }

  /** 序列化为与后端兼容的蛇形 JSON */
  toJSON() {
    return {
      source_name: this.sourceName,
      source_type: this.sourceType,
      entity_type: this.entityType,
      entity_id: this.entityId,
    };
  }
}
