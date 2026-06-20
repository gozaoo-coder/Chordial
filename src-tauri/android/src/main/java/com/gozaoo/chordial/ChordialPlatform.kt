package com.gozaoo.chordial

import android.app.Activity
import android.content.ContentUris
import android.content.Context
import android.database.Cursor
import android.net.Uri
import android.os.Environment
import android.provider.MediaStore
import android.provider.OpenableColumns
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import org.json.JSONArray
import org.json.JSONObject
import java.io.File

/**
 * Chordial 平台桥接插件 — 提供 Android 原生文件/媒体访问能力。
 *
 * 该插件在 Tauri 初始化时自动注册，Rust 端通过 JNI 调用其方法。
 * 同时也可通过 Tauri Command 从前端调用。
 *
 * ## 提供的功能
 * - 通过 ContentResolver 读取 content URI 文件的字节
 * - 查询 MediaStore 中的音频文件列表
 * - 获取文件大小和元数据
 * - SAF 文件夹选择器（预留）
 */
class ChordialPlatform(private val activity: Activity) : Plugin(activity) {

    companion object {
        /** 音频文件扩展名过滤 */
        val AUDIO_EXTENSIONS = setOf(
            "mp3", "flac", "wav", "ogg", "oga", "opus",
            "m4a", "aac", "wma", "aiff", "aif", "caf"
        )

        /** MediaStore 音频文件投影 */
        val AUDIO_PROJECTION = arrayOf(
            MediaStore.Audio.Media._ID,
            MediaStore.Audio.Media.DISPLAY_NAME,
            MediaStore.Audio.Media.TITLE,
            MediaStore.Audio.Media.ARTIST,
            MediaStore.Audio.Media.ALBUM,
            MediaStore.Audio.Media.DURATION,
            MediaStore.Audio.Media.SIZE,
            MediaStore.Audio.Media.DATA,       // 文件路径（可能为 null）
            MediaStore.Audio.Media.DATE_MODIFIED
        )
    }

    init {
        // 向 Rust 端注册自身实例
        nativeInit(this)
    }

    /** JNI: 将当前插件实例传递给 Rust 端 */
    private external fun nativeInit(plugin: ChordialPlatform)

    // ── JNI 可调用方法 ──────────────────────────────────────────────

    /**
     * 通过 ContentResolver 读取文件字节。
     *
     * 支持 `content://` URI 和普通文件路径。
     *
     * @param uriOrPath 文件 URI 或绝对路径
     * @return 文件内容的字节数组
     */
    fun readFileBytes(uriOrPath: String): ByteArray {
        // 尝试作为 content URI 读取
        if (uriOrPath.startsWith("content://")) {
            return readContentBytes(Uri.parse(uriOrPath))
        }

        // 尝试作为普通文件路径读取
        val file = File(uriOrPath)
        if (file.exists() && file.canRead()) {
            return file.readBytes()
        }

        // 也尝试从 MediaStore 按路径查找
        return readContentBytes(findUriByPath(uriOrPath) ?: Uri.parse(uriOrPath))
    }

    /**
     * 获取文件大小（字节）。
     */
    fun getFileSize(uriOrPath: String): Long {
        if (uriOrPath.startsWith("content://")) {
            return queryContentSize(Uri.parse(uriOrPath))
        }
        val file = File(uriOrPath)
        return if (file.exists()) file.length() else -1L
    }

    /**
     * 检查文件是否存在。
     */
    fun fileExists(uriOrPath: String): Boolean {
        if (uriOrPath.startsWith("content://")) {
            return try {
                val cursor = activity.contentResolver.query(
                    Uri.parse(uriOrPath), null, null, null, null
                )
                cursor?.use { it.count > 0 } ?: false
            } catch (_: Exception) {
                false
            }
        }
        return File(uriOrPath).exists()
    }

    /**
     * 查询指定文件夹（或全局）下的音频文件。
     *
     * @param folderUri 文件夹 URI 或 "all" 表示查询全局
     * @return JSON 数组字符串，每个元素包含音频文件的元数据
     */
    fun queryAudioFiles(folderUri: String): String {
        val results = JSONArray()

        try {
            val uri = MediaStore.Audio.Media.EXTERNAL_CONTENT_URI
            val selection: String?
            val selectionArgs: Array<String>?

            if (folderUri != "all" && folderUri.startsWith("content://")) {
                // 按文件夹过滤（通过父目录 ID）
                selection = "${MediaStore.Audio.Media.BUCKET_ID} = ?"
                selectionArgs = arrayOf(folderUri.substringAfterLast("/"))
            } else {
                selection = null
                selectionArgs = null
            }

            val cursor: Cursor? = activity.contentResolver.query(
                uri,
                AUDIO_PROJECTION,
                selection,
                selectionArgs,
                "${MediaStore.Audio.Media.DATE_MODIFIED} DESC"
            )

            cursor?.use {
                while (it.moveToNext()) {
                    val obj = JSONObject()
                    obj.put("id", it.getLong(0))
                    obj.put("displayName", it.getString(1) ?: "")
                    obj.put("title", it.getString(2) ?: "")
                    obj.put("artist", it.getString(3) ?: "")
                    obj.put("album", it.getString(4) ?: "")
                    obj.put("duration", it.getLong(5))
                    obj.put("size", it.getLong(6))
                    obj.put("data", it.getString(7) ?: "")
                    obj.put("dateModified", it.getLong(8))
                    obj.put("contentUri", ContentUris.withAppendedId(
                        MediaStore.Audio.Media.EXTERNAL_CONTENT_URI, it.getLong(0)
                    ).toString())
                    results.put(obj)
                }
            }
        } catch (e: Exception) {
            e.printStackTrace()
        }

        return results.toString()
    }

    // ── 内部辅助方法 ──────────────────────────────────────────────

    /**
     * 通过 ContentResolver 读取 content URI 文件的全部字节。
     */
    private fun readContentBytes(uri: Uri): ByteArray {
        activity.contentResolver.openInputStream(uri)?.use { input ->
            return input.readBytes()
        } ?: throw IllegalStateException("无法打开 content URI: $uri")
    }

    /**
     * 查询 content URI 对应的文件大小。
     */
    private fun queryContentSize(uri: Uri): Long {
        val cursor = activity.contentResolver.query(uri, null, null, null, null)
        cursor?.use {
            if (it.moveToFirst()) {
                val sizeIndex = it.getColumnIndex(OpenableColumns.SIZE)
                if (sizeIndex >= 0) {
                    return it.getLong(sizeIndex)
                }
            }
        }
        return -1L
    }

    /**
     * 按文件路径反查 MediaStore content URI。
     */
    private fun findUriByPath(path: String): Uri? {
        val uri = MediaStore.Audio.Media.EXTERNAL_CONTENT_URI
        val cursor = activity.contentResolver.query(
            uri,
            arrayOf(MediaStore.Audio.Media._ID),
            "${MediaStore.Audio.Media.DATA} = ?",
            arrayOf(path),
            null
        )
        cursor?.use {
            if (it.moveToFirst()) {
                val id = it.getLong(0)
                return ContentUris.withAppendedId(uri, id)
            }
        }
        return null
    }
}
