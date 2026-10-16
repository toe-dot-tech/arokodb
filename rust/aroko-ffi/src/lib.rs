use aroko_core::index::{DistanceMetric, IndexParams, VectorIndex};
use jni::objects::{JFloatArray, JLongArray, JObject};
use jni::sys::{jboolean, jfloatArray, jint, jlongArray};
use jni::JNIEnv;
use std::sync::{Mutex, OnceLock};

static INDEX: OnceLock<Mutex<Option<VectorIndex>>> = OnceLock::new();

fn index() -> &'static Mutex<Option<VectorIndex>> {
    INDEX.get_or_init(|| Mutex::new(None))
}

fn throw(env: &mut JNIEnv<'_>, message: &str) {
    let _ = env.throw_new("java/lang/IllegalStateException", message);
}

fn read_floats(env: &mut JNIEnv<'_>, raw: jfloatArray) -> Result<Vec<f32>, String> {
    if raw.is_null() {
        return Err("Vector array was null.".to_owned());
    }

    let array = unsafe { JFloatArray::from_raw(raw) };
    let length = env
        .get_array_length(&array)
        .map_err(|error| error.to_string())?;
    let mut values = vec![0.0; length as usize];
    env.get_float_array_region(&array, 0, &mut values)
        .map_err(|error| error.to_string())?;
    if values.iter().any(|value| !value.is_finite()) {
        return Err("Vector values must be finite.".to_owned());
    }
    Ok(values)
}

#[no_mangle]
pub extern "system" fn Java_com_arokodb_ArokoDB_nativeInitialize(
    mut env: JNIEnv<'_>,
    _this: JObject<'_>,
    dimension: jint,
) -> jboolean {
    if !(1..=4096).contains(&dimension) {
        throw(&mut env, "Dimension must be between 1 and 4096.");
        return 0;
    }

    match index().lock() {
        Ok(mut guard) => {
            *guard = Some(VectorIndex::new(IndexParams {
                dim: dimension as usize,
            }));
            1
        }
        Err(_) => {
            throw(&mut env, "The native index lock is unavailable.");
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_arokodb_ArokoDB_nativeInsert(
    mut env: JNIEnv<'_>,
    _this: JObject<'_>,
    id: jint,
    raw_vector: jfloatArray,
) -> jboolean {
    if id < 0 {
        throw(&mut env, "Vector ID must be non-negative.");
        return 0;
    }
    let vector = match read_floats(&mut env, raw_vector) {
        Ok(vector) => vector,
        Err(message) => {
            throw(&mut env, &message);
            return 0;
        }
    };

    match index().lock() {
        Ok(mut guard) => match guard.as_mut() {
            Some(index) => match index.insert_with_id(id as usize, vector) {
                Ok(()) => 1,
                Err(message) => {
                    throw(&mut env, &message);
                    0
                }
            },
            None => {
                throw(&mut env, "Create the index before adding vectors.");
                0
            }
        },
        Err(_) => {
            throw(&mut env, "The native index lock is unavailable.");
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_arokodb_ArokoDB_nativeSearch(
    mut env: JNIEnv<'_>,
    _this: JObject<'_>,
    raw_query: jfloatArray,
    k: jint,
    metric: jint,
) -> jlongArray {
    if k <= 0 {
        throw(&mut env, "Top K must be positive.");
        return std::ptr::null_mut();
    }
    let query = match read_floats(&mut env, raw_query) {
        Ok(query) => query,
        Err(message) => {
            throw(&mut env, &message);
            return std::ptr::null_mut();
        }
    };
    let metric = match metric {
        0 => DistanceMetric::SquaredEuclidean,
        1 => DistanceMetric::Cosine,
        2 => DistanceMetric::Manhattan,
        _ => {
            throw(&mut env, "Unsupported distance metric.");
            return std::ptr::null_mut();
        }
    };

    let encoded = match index().lock() {
        Ok(guard) => match guard.as_ref() {
            Some(index) if query.len() == index.params().dim => {
                let results = index.search_with_metric(&query, k as usize, metric);
                let mut encoded = Vec::with_capacity(results.len() * 2);
                for (id, distance) in results {
                    encoded.push(id as i64);
                    encoded.push(distance.to_bits() as u32 as i64);
                }
                encoded
            }
            Some(index) => {
                throw(
                    &mut env,
                    &format!(
                        "Query dimension {} does not match index dimension {}.",
                        query.len(),
                        index.params().dim
                    ),
                );
                return std::ptr::null_mut();
            }
            None => {
                throw(&mut env, "Create the index before searching.");
                return std::ptr::null_mut();
            }
        },
        Err(_) => {
            throw(&mut env, "The native index lock is unavailable.");
            return std::ptr::null_mut();
        }
    };

    let array = match env.new_long_array(encoded.len() as jint) {
        Ok(array) => array,
        Err(error) => {
            throw(&mut env, &error.to_string());
            return std::ptr::null_mut();
        }
    };
    if let Err(error) = env.set_long_array_region(&array, 0, &encoded) {
        throw(&mut env, &error.to_string());
        return std::ptr::null_mut();
    }
    let raw: jlongArray = JLongArray::into_raw(array);
    raw
}

#[no_mangle]
pub extern "system" fn Java_com_arokodb_ArokoDB_nativeSize(
    mut env: JNIEnv<'_>,
    _this: JObject<'_>,
) -> jint {
    match index().lock() {
        Ok(guard) => match guard.as_ref() {
            Some(index) => index.len().min(jint::MAX as usize) as jint,
            None => {
                throw(&mut env, "Create the index first.");
                0
            }
        },
        Err(_) => {
            throw(&mut env, "The native index lock is unavailable.");
            0
        }
    }
}
