//! Android 原生桥接 — 用 JNI 在 Rust 端调用 Kotlin 层的文件/媒体 API。
//!
//! 当 `android.rs` 中的 `std::fs` 无法访问 content URI 时，回退到此桥接层。
//! 通过 `jni` crate 直接调用 Kotlin 方法，绕过 JSON 序列化开销。
//!
//! # 架构
//!
//! ```text
//! Rust (android.rs) → android_bridge.rs → JNI → ChordialPlatform.kt → ContentResolver
//! ```

#![cfg(target_os = "android")]

use jni::objects::{GlobalRef, JByteArray, JClass, JObject, JValue};
use jni::JNIEnv;
use std::sync::OnceLock;

static JVM: OnceLock<jni::JavaVM> = OnceLock::new();
static PLUGIN_INSTANCE: OnceLock<GlobalRef> = OnceLock::new();

/// 初始化 JNI 桥接（由 Kotlin 端在插件初始化时调用）。
///
/// 从 Kotlin 端接收当前 `ChordialPlatform` 实例的全局引用，
/// 之后所有 Rust→Kotlin 调用都通过此引用进行。
#[no_mangle]
pub extern "system" fn Java_com_gozaoo_chordial_ChordialPlatform_nativeInit(
    env: JNIEnv,
    _class: JClass,
    plugin_instance: JObject,
) {
    let jvm = env.get_java_vm().expect("获取 JavaVM 失败");
    let _ = JVM.set(jvm);

    let global_ref = env
        .new_global_ref(plugin_instance)
        .expect("创建全局引用失败");
    let _ = PLUGIN_INSTANCE.set(global_ref);
}

/// 获取 JNI 环境（永久附加当前线程到 JVM）。
fn get_env() -> Result<JNIEnv<'static>, String> {
    let jvm = JVM.get().ok_or("JVM 未初始化")?;
    let env = jvm
        .attach_current_thread_permanently()
        .map_err(|e| format!("附加 JNI 线程失败: {}", e))?;
    Ok(env)
}

/// 通过 JNI 调用 Kotlin 的 `readFileBytes(contentUri: String): ByteArray`。
pub fn read_content_uri_bytes(uri: &str) -> Result<Vec<u8>, String> {
    let mut env = get_env()?;
    let instance = PLUGIN_INSTANCE
        .get()
        .ok_or("插件实例未初始化")?
        .as_obj();

    let uri_jstr = env
        .new_string(uri)
        .map_err(|e| format!("创建 JNI 字符串失败: {}", e))?;

    let result = env
        .call_method(
            instance,
            "readFileBytes",
            "(Ljava/lang/String;)[B",
            &[JValue::Object(&uri_jstr)],
        )
        .map_err(|e| format!("调用 readFileBytes 失败: {}", e))?;

    let obj = result
        .l()
        .map_err(|e| format!("获取返回值失败: {}", e))?;

    let byte_array: JByteArray = obj.into();
    let bytes = env
        .convert_byte_array(&byte_array)
        .map_err(|e| format!("转换字节数组失败: {}", e))?;

    Ok(bytes)
}

/// 通过 JNI 调用 Kotlin 的 `getFileSize(contentUri: String): Long`。
pub fn read_content_uri_size(uri: &str) -> Result<u64, String> {
    let mut env = get_env()?;
    let instance = PLUGIN_INSTANCE
        .get()
        .ok_or("插件实例未初始化")?
        .as_obj();

    let uri_jstr = env
        .new_string(uri)
        .map_err(|e| format!("创建 JNI 字符串失败: {}", e))?;

    let result = env
        .call_method(
            instance,
            "getFileSize",
            "(Ljava/lang/String;)J",
            &[JValue::Object(&uri_jstr)],
        )
        .map_err(|e| format!("调用 getFileSize 失败: {}", e))?;

    Ok(result.j().map_err(|e| format!("获取返回值失败: {}", e))? as u64)
}

/// 通过 JNI 调用 Kotlin 的 `fileExists(contentUri: String): Boolean`。
pub fn check_content_uri_exists(uri: &str) -> Result<bool, String> {
    let mut env = get_env()?;
    let instance = PLUGIN_INSTANCE
        .get()
        .ok_or("插件实例未初始化")?
        .as_obj();

    let uri_jstr = env
        .new_string(uri)
        .map_err(|e| format!("创建 JNI 字符串失败: {}", e))?;

    let result = env
        .call_method(
            instance,
            "fileExists",
            "(Ljava/lang/String;)Z",
            &[JValue::Object(&uri_jstr)],
        )
        .map_err(|e| format!("调用 fileExists 失败: {}", e))?;

    Ok(result.z().map_err(|e| format!("获取返回值失败: {}", e))?)
}

/// 通过 JNI 调用 Kotlin 的 `queryAudioFiles(folderUri: String): String`。
///
/// 返回 JSON 数组字符串，由调用者解析。
pub fn query_audio_files(uri: &str) -> Result<String, String> {
    let mut env = get_env()?;
    let instance = PLUGIN_INSTANCE
        .get()
        .ok_or("插件实例未初始化")?
        .as_obj();

    let uri_jstr = env
        .new_string(uri)
        .map_err(|e| format!("创建 JNI 字符串失败: {}", e))?;

    let result = env
        .call_method(
            instance,
            "queryAudioFiles",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(&uri_jstr)],
        )
        .map_err(|e| format!("调用 queryAudioFiles 失败: {}", e))?;

    let jstr: jni::objects::JString = result
        .l()
        .map_err(|e| format!("获取返回值失败: {}", e))?
        .into();

    let rust_str: String = env
        .get_string(&jstr)
        .map_err(|e| format!("读取 JNI 字符串失败: {}", e))?
        .into();

    Ok(rust_str)
}
