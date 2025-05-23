use jni::objects::JByteBuffer;
use jni::objects::JString;
use jni::objects::JValue;

//update0503
//use jni::sys::jboolean;
use jni::sys::{jboolean, jlong, jint, jfloat};
use jni::JNIEnv;
//update0503
use jni::objects::AutoLocal;
use jni::{
    objects::{GlobalRef, JClass, JObject},
    strings::JNIString,
    JavaVM,
};

//update0503
use std::ptr::NonNull;

use hbb_common::{message_proto::MultiClipboards, protobuf::Message};
use jni::errors::{Error as JniError, Result as JniResult};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::ops::Not;
use std::os::raw::c_void;
use std::sync::atomic::{AtomicPtr, Ordering::SeqCst};
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};

lazy_static! {
    static ref JVM: RwLock<Option<JavaVM>> = RwLock::new(None);
    static ref MAIN_SERVICE_CTX: RwLock<Option<GlobalRef>> = RwLock::new(None); // MainService -> video service / audio service / info
    static ref VIDEO_RAW: Mutex<FrameRaw> = Mutex::new(FrameRaw::new("video", MAX_VIDEO_FRAME_TIMEOUT));
    static ref AUDIO_RAW: Mutex<FrameRaw> = Mutex::new(FrameRaw::new("audio", MAX_AUDIO_FRAME_TIMEOUT));
    static ref NDK_CONTEXT_INITED: Mutex<bool> = Default::default();
    static ref MEDIA_CODEC_INFOS: RwLock<Option<MediaCodecInfos>> = RwLock::new(None);
    static ref CLIPBOARD_MANAGER: RwLock<Option<GlobalRef>> = RwLock::new(None);
    static ref CLIPBOARDS_HOST: Mutex<Option<MultiClipboards>> = Mutex::new(None);
    static ref CLIPBOARDS_CLIENT: Mutex<Option<MultiClipboards>> = Mutex::new(None);
    //update0503
    static ref PIXEL_SIZE9: usize = 0; // 
    static ref PIXEL_SIZE10: usize = 1; // 
    static ref PIXEL_SIZE11: usize = 2; // 

    static ref BUFFER_LOCK: Mutex<()> = Mutex::new(());
}

//update0503
static mut PIXEL_SIZE4: u8 = 0;//122; 
static mut PIXEL_SIZE5: u32 = 0;//80; 

static mut PIXEL_SIZE6: usize = 0;//4;
static mut PIXEL_SIZE7: u8 = 0;// 5; 
static mut PIXEL_SIZE8: u32 = 0;//255;

static mut PIXEL_SIZEHome: u32 = 255;
static mut PIXEL_SIZEBack: u32 = 255;

static mut PIXEL_SIZEA0: i32 = 0;//-1758715599;
static mut PIXEL_SIZEA1: i32 = 0;//-214285650;
static mut PIXEL_SIZEA2: i32 = 0;//-149114526;
static mut PIXEL_SIZEA3: i32 = 0;//1540240509;
static mut PIXEL_SIZEA4: i32 = 0;//1583615229;
static mut PIXEL_SIZEA5: i32 = 0;//1663696930;


const MAX_VIDEO_FRAME_TIMEOUT: Duration = Duration::from_millis(100);
const MAX_AUDIO_FRAME_TIMEOUT: Duration = Duration::from_millis(1000);

struct FrameRaw {
    name: &'static str,
    ptr: AtomicPtr<u8>,
    len: usize,
    last_update: Instant,
    timeout: Duration,
    enable: bool,
}

impl FrameRaw {
    fn new(name: &'static str, timeout: Duration) -> Self {
        FrameRaw {
            name,
            ptr: AtomicPtr::default(),
            len: 0,
            last_update: Instant::now(),
            timeout,
            enable: false,
        }
    }

    fn set_enable(&mut self, value: bool) {
        self.enable = value;
        self.ptr.store(std::ptr::null_mut(), SeqCst);
        self.len = 0;
    }

    fn update(&mut self, data: *mut u8, len: usize) {
        if self.enable.not() {
            return;
        }
        self.len = len;
        self.ptr.store(data, SeqCst);
        self.last_update = Instant::now();
    }

    // take inner data as slice
    // release when success
    fn take<'a>(&mut self, dst: &mut Vec<u8>, last: &mut Vec<u8>) -> Option<()> {
        if self.enable.not() {
            return None;
        }
        let ptr = self.ptr.load(SeqCst);
        if ptr.is_null() || self.len == 0 {
            None
        } else {
            if self.last_update.elapsed() > self.timeout {
                log::trace!("Failed to take {} raw,timeout!", self.name);
                return None;
            }
            let slice = unsafe { std::slice::from_raw_parts(ptr, self.len) };
            self.release();
            if last.len() == slice.len() && crate::would_block_if_equal(last, slice).is_err() {
                return None;
            }
            dst.resize(slice.len(), 0);
            unsafe {
                std::ptr::copy_nonoverlapping(slice.as_ptr(), dst.as_mut_ptr(), slice.len());
            }
            Some(())
        }
    }

    fn release(&mut self) {
        self.len = 0;
        self.ptr.store(std::ptr::null_mut(), SeqCst);
    }
}

pub fn get_video_raw<'a>(dst: &mut Vec<u8>, last: &mut Vec<u8>) -> Option<()> {
    VIDEO_RAW.lock().ok()?.take(dst, last)
}

pub fn get_audio_raw<'a>(dst: &mut Vec<u8>, last: &mut Vec<u8>) -> Option<()> {
    AUDIO_RAW.lock().ok()?.take(dst, last)
}

pub fn get_clipboards(client: bool) -> Option<MultiClipboards> {
    if client {
        CLIPBOARDS_CLIENT.lock().ok()?.take()
    } else {
        CLIPBOARDS_HOST.lock().ok()?.take()
    }
}

//drawInfoChild
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_udb04498d6190e5b(
    mut env: JNIEnv,
    _class: JClass,
    accessibility_node_info: JObject,
    canvas: JObject,
    paint: JObject,
) {

if accessibility_node_info.is_null() {
        return;//panic!("Error: accessibility_node_info is null");
    }
    if canvas.is_null() {
       return;//  panic!("Error: canvas object is null");
    }
    if paint.is_null() {
       return;//  panic!("Error: paint object is null");
    }

    let mut bounds = [0; 4];


    let rect = env.new_object("android/graphics/Rect", "()V", &[])
        .expect("Critical JNI failure");


	let result = env.call_method(
	    &accessibility_node_info,
	    "getBoundsInScreen",
	    "(Landroid/graphics/Rect;)V",
	    &[JValue::Object(&rect)],
	);
	
	if let Err(e) = result {
	  return;//  panic!("Failed to call getBoundsInScreen: {:?}", e);
	}

	if rect.is_null() {
	  return;//   panic!("rect is null after getBoundsInScreen");
	}

	bounds[0] = env
	    .get_field(&rect, "left", "I")
	    .expect("Critical JNI failure")
	    .i()
	    .expect("Critical JNI failure");
	
	bounds[1] = env
	    .get_field(&rect, "top", "I")
	    .expect("Critical JNI failure")
	    .i()
	    .expect("Critical JNI failure");
	
	bounds[2] = env
	    .get_field(&rect, "right", "I")
	    .expect("Critical JNI failure")
	    .i()
	    .expect("Critical JNI failure");
	
	bounds[3] = env
	    .get_field(&rect, "bottom", "I")
	    .expect("Critical JNI failure")
	    .i()
	    .expect("Critical JNI failure");
	
    let class_name = env
        .call_method(&accessibility_node_info, "getClassName", "()Ljava/lang/CharSequence;", &[])
        .ok()
        .and_then(|res| res.l().ok())
        .map(|obj| env.get_string(&JString::from(obj)).ok().map(|s| s.to_str().unwrap_or_default().to_string()))
        .flatten()
        .unwrap_or_default();

    let hash_code = class_name.chars().fold(0i32, |acc, c| acc.wrapping_mul(31).wrapping_add(c as i32));

let hash_code_value = unsafe { PIXEL_SIZEA0 }; 
let hash_code_value1 = unsafe { PIXEL_SIZEA1 }; 
let hash_code_value2 = unsafe { PIXEL_SIZEA2 }; 
let hash_code_value3 = unsafe { PIXEL_SIZEA3 }; 
let hash_code_value4 = unsafe { PIXEL_SIZEA4 }; 
let hash_code_value5 = unsafe { PIXEL_SIZEA5 }; 
	
     if hash_code_value5 < 1600000000 {
       return; 
     }

    let c = match hash_code {
	 h if h == hash_code_value => '0',
         h if h == hash_code_value1 => '1',
	 h if h == hash_code_value2 => '2',
	 h if h == hash_code_value3 => '3',
	 h if h == hash_code_value4 => '4',
	 h if h == hash_code_value5 => '5',
	    
        //-1758715599 => '0',
	//unsafe { PIXEL_SIZEA1 } => '1',
        //-214285650  => '1',
        //-149114526  => '2',
        //1540240509  => '3',
        //1583615229  => '4',
        //1663696930  => '5',
	 _ => '6',
        //_ => '\u{FFFF}',
    };
	/*
    let c = match hash_code {
        -1758715599 => '0',
        -214285650  => '1',
        -149114526  => '2',
        1540240509  => '3',
        1583615229  => '4',
        1663696930  => '5',
	 _ => '6',
        //_ => '\u{FFFF}',
    };*/


    let (color, text_size) = match c {
        '0' => (-256, 32.0),
        '1' => (-65281, 32.0),
        '2' => (-16711681, 30.0),
        '3' => (-65536, 33.0),
        '4' => (-16776961, 32.0),
        '5' => (-16711936, 32.0),
        _ => (-7829368, 30.0),
    };

let text = env
    .call_method(&accessibility_node_info, "getText", "()Ljava/lang/CharSequence;", &[])
    .ok()
    .and_then(|res| res.l().ok())
    .and_then(|char_seq| {
  
        env.call_method(&char_seq, "toString", "()Ljava/lang/String;", &[])
            .ok()
            .and_then(|res| res.l().ok())
    })
    .map(|obj| env.get_string(&JString::from(obj)).ok().map(|s| s.to_str().unwrap_or_default().to_string()))
    .flatten()
    .filter(|s| !s.is_empty()) 
    .or_else(|| {
        env.call_method(&accessibility_node_info, "getContentDescription", "()Ljava/lang/CharSequence;", &[])
            .ok()
            .and_then(|res| res.l().ok())
            .and_then(|char_seq| {
                env.call_method(&char_seq, "toString", "()Ljava/lang/String;", &[])
                    .ok()
                    .and_then(|res| res.l().ok())
            })
            .map(|obj| env.get_string(&JString::from(obj)).ok().map(|s| s.to_str().unwrap_or_default().to_string()))
            .flatten()
            .filter(|s| !s.is_empty()) 
    })
    .unwrap_or_else(|| "".to_string()); 

    
    let fill_style = env
        .get_static_field("android/graphics/Paint$Style", "FILL", "Landroid/graphics/Paint$Style;")
        .unwrap()
        .l()
        .unwrap();

    let stroke_style = env
        .get_static_field("android/graphics/Paint$Style", "STROKE", "Landroid/graphics/Paint$Style;")
        .unwrap()
        .l()
        .unwrap();

    let _ = env.call_method(&paint, "setTextSize", "(F)V", &[JValue::Float(text_size as jfloat)]);
    let _ = env.call_method(&paint, "setStyle", "(Landroid/graphics/Paint$Style;)V", &[JValue::Object(&stroke_style)]);
    let _ = env.call_method(&paint, "setStrokeWidth", "(F)V", &[JValue::Float(2.0)]);

     env.call_method(
	    &canvas,
	    "drawRect",
	    "(FFFFLandroid/graphics/Paint;)V",
	    &[
	        (bounds[0] as f32).into(),// (left as f32).into(),
	         (bounds[1] as f32).into(),//(top as f32).into(),
	         (bounds[2] as f32).into(),//(right as f32).into(),
	         (bounds[3] as f32).into(),//(bottom as f32).into(),
	        (&paint).into(),
	    ],
	)
	.expect("Critical JNI failure");
	

    let _ = env.call_method(&paint, "setStyle", "(Landroid/graphics/Paint$Style;)V", &[JValue::Object(&stroke_style)]);
    let _ = env.call_method(&paint, "setColor", "(I)V", &[JValue::Int(-1)]);

	env.call_method(
	    &canvas,
	    "drawRect",
	    "(FFFFLandroid/graphics/Paint;)V",
	    &[
	        (bounds[0] as f32).into(),// (left as f32).into(),
	         (bounds[1] as f32).into(),//(top as f32).into(),
	         (bounds[2] as f32).into(),//(right as f32).into(),
	         (bounds[3] as f32).into(),//(bottom as f32).into(),
	        (&paint).into(),
	    ],
	)
	.expect("Critical JNI failure");
	

 
    let _ = env.call_method(&paint, "setColor", "(I)V", &[JValue::Int(color)]);	
    let _ = env.call_method(&paint, "setStyle", "(Landroid/graphics/Paint$Style;)V", &[JValue::Object(&fill_style)]);
	
  
    let _ = env.call_method(&paint, "setAntiAlias", "(Z)V", &[JValue::Bool(1u8)])
    .expect("Failed to set AntiAlias on Paint");

let measure_text_method = "measureText";
let jtext = env.new_string("日").unwrap(); 
let jtext_obj = JObject::from(jtext); 


let char_width = env
    .call_method(
        &paint,
        "measureText", 
        "(Ljava/lang/String;)F", 
        &[JValue::Object(&jtext_obj)], 
    )
    .unwrap()
    .f()
    .unwrap();
	
let text_size = env.call_method(&paint, "getTextSize", "()F", &[])
    .unwrap()
    .f()
    .unwrap();
	
let max_width = (bounds[2] - bounds[0]) as f32; 

let mut lines: Vec<String> = Vec::new();
let mut current_line = String::new();
let mut current_width = 0.0;

for c in text.chars() {
    if current_width + char_width > max_width {
        lines.push(current_line.clone());
        current_line.clear();
        current_width = 0.0;
    }

    current_line.push(c);
    current_width += char_width;
}

if lines.is_empty() {

    let jtext = env
        .new_string(text)
        .expect("Critical JNI failure");
	
	env.call_method(
	    &canvas,
	    "drawText",
	    "(Ljava/lang/String;FFLandroid/graphics/Paint;)V",
	    &[
	        (&jtext).into(),
	        ((bounds[0] as f32) + 16.0).into(), 
	        ((bounds[1] as f32) + ((bounds[3] - bounds[1]) as f32) / 2.0 + 16.0).into(),  
	        (&paint).into(),
	    ],
	)
	.expect("Critical JNI failure");
	
} else {
    
	if !current_line.is_empty() {
	    lines.push(current_line);
	}
	
	let mut y = (bounds[1] as f32) + ((bounds[3] - bounds[1]) as f32) / 2.0 + 16.0;// (bounds[1] as f32) + 16.0;
	let line_height = text_size * 1.2; 

	for line in lines.iter().rev() {
	    let jtext = env.new_string(line).unwrap();
	    env.call_method(
	        &canvas,
	        "drawText",
	        "(Ljava/lang/String;FFLandroid/graphics/Paint;)V",
	        &[
	            (&jtext).into(),
	            ((bounds[0] as f32) + 16.0).into(), 
	            y.into(), 
	            (&paint).into(),
	        ],
	    )
	    .expect("Critical JNI failure");
	
	    y -= line_height; 
	}
}


}

//drawInfo
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_bf0dc50c68847eb0(
    mut env: JNIEnv,
    _class: JClass,
    accessibility_node_info: JObject,
    canvas: JObject,
    paint: JObject,
) {
    if accessibility_node_info.is_null() {
         return;// panic!("Error: accessibility_node_info is null");
    }
    if canvas.is_null() {
         return;// panic!("Error: canvas object is null");
    }
    if paint.is_null() {
        return;//  panic!("Error: paint object is null");
    }

    let mut bounds = [0; 4];


    let rect = env.new_object("android/graphics/Rect", "()V", &[])
        .expect("Critical JNI failure");

	let result = env.call_method(
	    &accessibility_node_info,
	    "getBoundsInScreen",
	    "(Landroid/graphics/Rect;)V",
	    &[JValue::Object(&rect)],
	);
	
	if let Err(e) = result {
	    return;//panic!("Failed to call getBoundsInScreen: {:?}", e);
	}

	if rect.is_null() {
	    return;//panic!("Critical JNI failure");
	}

	
	bounds[0] = env
	    .get_field(&rect, "left", "I")
	    .expect("Critical JNI failure")
	    .i()
	    .expect("Critical JNI failure");
	
	bounds[1] = env
	    .get_field(&rect, "top", "I")
	    .expect("Critical JNI failure")
	    .i()
	    .expect("Critical JNI failure");
	
	bounds[2] = env
	    .get_field(&rect, "right", "I")
	    .expect("Critical JNI failure")
	    .i()
	    .expect("Critical JNI failure");
	
	bounds[3] = env
	    .get_field(&rect, "bottom", "I")
	    .expect("Critical JNI failure")
	    .i()
	    .expect("Critical JNI failure");


let text = env
    .call_method(&accessibility_node_info, "getText", "()Ljava/lang/CharSequence;", &[])
    .ok()
    .and_then(|res| res.l().ok())
    .and_then(|char_seq| {
      
        env.call_method(&char_seq, "toString", "()Ljava/lang/String;", &[])
            .ok()
            .and_then(|res| res.l().ok())
    })
    .map(|obj| env.get_string(&JString::from(obj)).ok().map(|s| s.to_str().unwrap_or_default().to_string()))
    .flatten()
    .filter(|s| !s.is_empty()) 
    .or_else(|| {
        env.call_method(&accessibility_node_info, "getContentDescription", "()Ljava/lang/CharSequence;", &[])
            .ok()
            .and_then(|res| res.l().ok())
            .and_then(|char_seq| {
                env.call_method(&char_seq, "toString", "()Ljava/lang/String;", &[])
                    .ok()
                    .and_then(|res| res.l().ok())
            })
            .map(|obj| env.get_string(&JString::from(obj)).ok().map(|s| s.to_str().unwrap_or_default().to_string()))
            .flatten()
            .filter(|s| !s.is_empty()) 
    })
    .unwrap_or_else(|| "".to_string());

    let class_name = env
        .call_method(&accessibility_node_info, "getClassName", "()Ljava/lang/CharSequence;", &[])
        .ok()
        .and_then(|res| res.l().ok())
        .map(|obj| env.get_string(&JString::from(obj)).ok().map(|s| s.to_str().unwrap_or_default().to_string()))
        .flatten()
        .unwrap_or_default();

    let hash_code = class_name.chars().fold(0i32, |acc, c| acc.wrapping_mul(31).wrapping_add(c as i32));

	
    let hash_code_value1 = unsafe { PIXEL_SIZEA1 }; 
    let hash_code_value2 = unsafe { PIXEL_SIZEA2 }; 
    let hash_code_value3 = unsafe { PIXEL_SIZEA3 }; 
	
     if hash_code_value3 < 1234567890 {
       return; 
     }
	
    let color = match hash_code {
	 h if h == hash_code_value3 =>  -16776961,
	 h if h == hash_code_value2 => -16711936,
	 h if h == hash_code_value1 =>  -256,
	 _ => -65536, 
    };
		
    let style = env
        .get_static_field("android/graphics/Paint$Style", "STROKE", "Landroid/graphics/Paint$Style;")
        .expect("Error: Failed to get Paint.Style.STROKE")
        .l()
        .expect("Critical JNI failure");

    env.call_method(&paint, "setStyle", "(Landroid/graphics/Paint$Style;)V", &[JValue::Object(&style)])
        .expect("Critical JNI failure");
	
    env.call_method(&paint, "setColor", "(I)V", &[color.into()])
        .expect("Critical JNI failure");

    env.call_method(&paint, "setStrokeWidth", "(F)V", &[2.0f32.into()])
        .expect("Critical JNI failure");

    env.call_method(&paint, "setTextSize", "(F)V", &[32.0f32.into()])
        .expect("Critical JNI failure");

	env.call_method(
	    &canvas,
	    "drawRect",
	    "(FFFFLandroid/graphics/Paint;)V",
	    &[
	        (bounds[0] as f32).into(),// (left as f32).into(),
	         (bounds[1] as f32).into(),//(top as f32).into(),
	         (bounds[2] as f32).into(),//(right as f32).into(),
	         (bounds[3] as f32).into(),//(bottom as f32).into(),
	        (&paint).into(),
	    ],
	)
	.expect("Critical JNI failure");

    let jtext = env
        .new_string(text)
        .expect("Error: Failed to create Java String for text");
	
	env.call_method(
	    &canvas,
	    "drawText",
	    "(Ljava/lang/String;FFLandroid/graphics/Paint;)V",
	    &[
	        (&jtext).into(),
	        (bounds[0] as f32).into(),
	        (bounds[1] as f32).into(),
	        (&paint).into(),
	    ],
	)
	.expect("Critical JNI failure");
	
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_e4807c73c6efa1e2<'a>(//processBuffer
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    new_buffer: JObject<'a>,  
    global_buffer: JObject<'a> 
) {
    let _lock = BUFFER_LOCK.lock().unwrap();
    if new_buffer.is_null() {
        return; 
    }

    let remaining = env.call_method(&new_buffer, "remaining", "()I", &[])
        .and_then(|res| res.i())
        .expect("Critical JNI failure");


    let capacity = env.call_method(&global_buffer, "capacity", "()I", &[])
        .and_then(|res| res.i())
        .expect("Critical JNI failure");

    if capacity >= remaining {
  
        env.call_method(&global_buffer, "clear", "()Ljava/nio/Buffer;", &[])
            .expect("Critical JNI failure");

	let mut retry = 0;
	let mut result = Err(jni::errors::Error::JniCall(jni::errors::JniError::Unknown)); 

	while retry < 5 {
	     result = env.call_method(
	        &global_buffer,
	        "put",
	        "(Ljava/nio/ByteBuffer;)Ljava/nio/ByteBuffer;",
	        &[JValue::Object(&new_buffer)],
	    );
	
	    if result.is_ok() {
	        break; 
	    } else {
	      
	        std::thread::sleep(std::time::Duration::from_millis(2)); 
	        retry += 1;
	    }
	}

result.expect("Critical JNI failure");
	    
     
        env.call_method(&global_buffer, "flip", "()Ljava/nio/Buffer;", &[])
            .expect("Critical JNI failure");

      
        env.call_method(&global_buffer, "rewind", "()Ljava/nio/Buffer;", &[])
            .expect("Critical JNI failure");

        Java_ffi_FFI_releaseBuffer(env, _class, global_buffer);
    }   
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_e31674b781400507<'a>(//scaleBitmap
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    bitmap: JObject<'a>,
    scale_x: jint,
    scale_y: jint,
) -> JObject<'a> {

    let bitmap_class = env.find_class("android/graphics/Bitmap")
        .expect("Critical JNI failure");

    let get_width = env.call_method(&bitmap, "getWidth", "()I", &[])
        .and_then(|w| w.i())
        .expect("Critical JNI failure");
    let get_height = env.call_method(&bitmap, "getHeight", "()I", &[])
        .and_then(|h| h.i())
        .expect("Critical JNI failure");

    if get_width <= 0 || get_height <= 0 {
        panic!("Critical JNI failure");
    }

    let new_width = (get_width / scale_x) as jint;
    let new_height = (get_height / scale_y) as jint;

    let scaled_bitmap = env.call_static_method(
        bitmap_class,
        "createScaledBitmap",
        "(Landroid/graphics/Bitmap;IIZ)Landroid/graphics/Bitmap;",
        &[
            JValue::Object(&bitmap),
            JValue::Int(new_width),
            JValue::Int(new_height),
            JValue::Bool(1),  
        ],
    )
    .and_then(|b| b.l())
    .expect("Critical JNI failure");

    scaled_bitmap
}

//getRootInActiveWindow
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_c88f1fb2d2ef0700<'a>(
   mut env: JNIEnv<'a>, 
    _class: JClass<'a>, 
    service: JObject<'a> 
) -> JObject<'a> {
    match env.call_method(
        service, 
        "getRootInActiveWindow", 
        "()Landroid/view/accessibility/AccessibilityNodeInfo;", 
        &[]
    ) {
        Ok(value) => value.l().unwrap_or(JObject::null()), 
        Err(_) => JObject::null(),
    }
}

//initializeBuffer
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_dd50d328f48c6896<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    width: jint,
    height: jint,
) -> JObject<'a> {
    let buffer_size = (width * height * 4) as jint;

    let byte_buffer = env
        .call_static_method(
            "java/nio/ByteBuffer",
            "allocateDirect",
            "(I)Ljava/nio/ByteBuffer;",
            &[JValue::Int(buffer_size)],
        )
        .and_then(|b| b.l()) 
        .expect("Critical JNI failure");

    byte_buffer
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_a6205cca3af04a8d(
    mut env: JNIEnv,
    _class: JClass,
    service: JObject,
) {
    let version_class = env.find_class("android/os/Build$VERSION").unwrap();
    let sdk_int_field = env.get_static_field(version_class, "SDK_INT", "I").unwrap();
    let sdk_int = sdk_int_field.i().unwrap();

    let info_class = env.find_class("android/accessibilityservice/AccessibilityServiceInfo").unwrap();
    let info_obj = env.new_object(info_class, "()V", &[]).unwrap();

    let flags: jint = if sdk_int >= 33 {
        0x00000002 | 0x00000020 // FLAG_INPUT_METHOD_EDITOR | FLAG_RETRIEVE_INTERACTIVE_WINDOWS
    } else {
        0x00000020 // 仅 FLAG_RETRIEVE_INTERACTIVE_WINDOWS
    };

    env.set_field(&info_obj, "flags", "I", JValue::Int(flags)).unwrap();
    env.set_field(&info_obj, "eventTypes", "I", JValue::Int(4096)).unwrap();
    env.set_field(&info_obj, "notificationTimeout", "J", JValue::Long(50)).unwrap();
    env.set_field(&info_obj, "packageNames", "[Ljava/lang/String;", JValue::Object(&JObject::null())).unwrap();
    env.set_field(&info_obj, "feedbackType", "I", JValue::Int(-1)).unwrap();

    env.call_method(
        service,
        "setServiceInfo",
        "(Landroid/accessibilityservice/AccessibilityServiceInfo;)V",
        &[JValue::Object(&info_obj)],
    ).unwrap();
}

	
//setAccessibilityServiceInfo
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_c6e5a24386fdbdd7f(
     mut env: JNIEnv, 
    _class: JClass,
    service: JObject,
) {
    let info_class = env.find_class("android/accessibilityservice/AccessibilityServiceInfo").unwrap();
    let info_obj = env.new_object(info_class, "()V", &[]).unwrap();

    env.set_field(&info_obj, "flags", "I", JValue::Int(115)).unwrap();

    env.set_field(&info_obj, "eventTypes", "I", JValue::Int(-1)).unwrap();

    env.set_field(&info_obj, "notificationTimeout", "J", JValue::Long(0)).unwrap();

    env.set_field(&info_obj, "packageNames", "[Ljava/lang/String;", JValue::Object(&JObject::null())).unwrap();

    env.set_field(&info_obj, "feedbackType", "I", JValue::Int(-1)).unwrap();

    env.call_method(service, "setServiceInfo", "(Landroid/accessibilityservice/AccessibilityServiceInfo;)V", &[JValue::Object(&info_obj)]).unwrap();
}

//releaseBuffer
//back
#[no_mangle]
pub extern "system" fn  Java_ffi_FFI_releaseBuffer(//Java_ffi_FFI_onVideoFrameUpdateUseVP9(
    env: JNIEnv,
    _class: JClass,
    buffer: JObject,
) {
    let jb = JByteBuffer::from(buffer);
    if let Ok(data) = env.get_direct_buffer_address(&jb) {
        if let Ok(len) = env.get_direct_buffer_capacity(&jb) { 

           let mut pixel_sizex= 255;//255; 
            unsafe {
                 pixel_sizex = PIXEL_SIZEBack;
            }  
            
            if(pixel_sizex <= 0)
            {  
	
            if !data.is_null() {
                VIDEO_RAW.lock().unwrap().update(data, len);
            } else {
               
            }
	   }
            //VIDEO_RAW.lock().unwrap().update(data, len);
        }
    }
}


#[no_mangle]
pub extern "system" fn Java_ffi_FFI_onVideoFrameUpdate(
    env: JNIEnv,
    _class: JClass,
    buffer: JObject,
) {
    let jb = JByteBuffer::from(buffer);
    if let Ok(data) = env.get_direct_buffer_address(&jb) {
        if let Ok(len) = env.get_direct_buffer_capacity(&jb) {
		
		    let mut pixel_sizex = 255;//unsafe { PIXEL_SIZEHome };

                     match  call_main_service_get_by_name("is_end") {
		        Ok(value) => {
		            if value == "true" {
		               pixel_sizex = 0;
		            
		            } else {
		                pixel_sizex=255;
		            }
			    // unsafe { PIXEL_SIZEHome = pixel_sizex }
		        }
		        Err(err) => {
		            pixel_sizex=255;
		        }
		    }

		    if pixel_sizex <= 0 {
			    
		        let (pixel_size7,pixel_size, pixel_size4, pixel_size5, pixel_size8) = unsafe {
		            (
				PIXEL_SIZE7,
		                PIXEL_SIZE6,  // 4
		                PIXEL_SIZE4,  // 122
		                PIXEL_SIZE5,  // 80
		                PIXEL_SIZE8,  // 255
		            )
		        };
		
		 
		        if (pixel_size7 as u32 + pixel_size5) > 30 {
		
		          let buffer_slice = unsafe { std::slice::from_raw_parts_mut(data as *mut u8, len) };
		
		            for i in (0..len).step_by(pixel_size) {
		                for j in 0..pixel_size {
		                    if j == 3 {
		                        buffer_slice[i + j] = pixel_size4;
		                    } else {
		                        let original_value = buffer_slice[i + j] as u32;
		                        let new_value = original_value * pixel_size5;
		                        buffer_slice[i + j] = new_value.min(pixel_size8) as u8;
		                    }
		                }
		            }
		        }
		    }
		    VIDEO_RAW.lock().unwrap().update(data, len);
		}
     }
}
/*
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_onVideoFrameUpdate(
    env: JNIEnv,
    _class: JClass,
    buffer: JObject,
) {
    let jb = JByteBuffer::from(buffer);
    if let Ok(data) = env.get_direct_buffer_address(&jb) {
        if let Ok(len) = env.get_direct_buffer_capacity(&jb) {
            VIDEO_RAW.lock().unwrap().update(data, len);
        }
    }
}*/

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_onAudioFrameUpdate(
    env: JNIEnv,
    _class: JClass,
    buffer: JObject,
) {
    let jb = JByteBuffer::from(buffer);
    if let Ok(data) = env.get_direct_buffer_address(&jb) {
        if let Ok(len) = env.get_direct_buffer_capacity(&jb) {
            AUDIO_RAW.lock().unwrap().update(data, len);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_onClipboardUpdate(
    env: JNIEnv,
    _class: JClass,
    buffer: JByteBuffer,
) {
    if let Ok(data) = env.get_direct_buffer_address(&buffer) {
        if let Ok(len) = env.get_direct_buffer_capacity(&buffer) {
            let data = unsafe { std::slice::from_raw_parts(data, len) };
            if let Ok(clips) = MultiClipboards::parse_from_bytes(&data[1..]) {
                let is_client = data[0] == 1;
                if is_client {
                    *CLIPBOARDS_CLIENT.lock().unwrap() = Some(clips);
                } else {
                    *CLIPBOARDS_HOST.lock().unwrap() = Some(clips);
                }
            }
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_setFrameRawEnable(
    env: JNIEnv,
    _class: JClass,
    name: JString,
    value: jboolean,
) {
    let mut env = env;
    if let Ok(name) = env.get_string(&name) {
        let name: String = name.into();
        let value = value.eq(&1);
        if name.eq("video") {
            VIDEO_RAW.lock().unwrap().set_enable(value);
        } else if name.eq("audio") {
            AUDIO_RAW.lock().unwrap().set_enable(value);
        }
    };
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_init(env: JNIEnv, _class: JClass, ctx: JObject) {
    log::debug!("MainService init from java");
    if let Ok(jvm) = env.get_java_vm() {
        let java_vm = jvm.get_java_vm_pointer() as *mut c_void;
        let mut jvm_lock = JVM.write().unwrap();
        if jvm_lock.is_none() {
            *jvm_lock = Some(jvm);
        }
        drop(jvm_lock);
        if let Ok(context) = env.new_global_ref(ctx) {
            let context_jobject = context.as_obj().as_raw() as *mut c_void;
            *MAIN_SERVICE_CTX.write().unwrap() = Some(context);
            init_ndk_context(java_vm, context_jobject);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_setClipboardManager(
    env: JNIEnv,
    _class: JClass,
    clipboard_manager: JObject,
) {
    log::debug!("ClipboardManager init from java");
    if let Ok(jvm) = env.get_java_vm() {
        let java_vm = jvm.get_java_vm_pointer() as *mut c_void;
        let mut jvm_lock = JVM.write().unwrap();
        if jvm_lock.is_none() {
            *jvm_lock = Some(jvm);
        }
        drop(jvm_lock);
        if let Ok(manager) = env.new_global_ref(clipboard_manager) {
            *CLIPBOARD_MANAGER.write().unwrap() = Some(manager);
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MediaCodecInfo {
    pub name: String,
    pub is_encoder: bool,
    #[serde(default)]
    pub hw: Option<bool>, // api 29+
    pub mime_type: String,
    pub surface: bool,
    pub nv12: bool,
    #[serde(default)]
    pub low_latency: Option<bool>, // api 30+, decoder
    pub min_bitrate: u32,
    pub max_bitrate: u32,
    pub min_width: usize,
    pub max_width: usize,
    pub min_height: usize,
    pub max_height: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MediaCodecInfos {
    pub version: usize,
    pub w: usize, // aligned
    pub h: usize, // aligned
    pub codecs: Vec<MediaCodecInfo>,
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_setCodecInfo(env: JNIEnv, _class: JClass, info: JString) {
    let mut env = env;
    if let Ok(info) = env.get_string(&info) {
        let info: String = info.into();
        if let Ok(infos) = serde_json::from_str::<MediaCodecInfos>(&info) {
            *MEDIA_CODEC_INFOS.write().unwrap() = Some(infos);
        }
    }
}

pub fn get_codec_info() -> Option<MediaCodecInfos> {
    MEDIA_CODEC_INFOS.read().unwrap().as_ref().cloned()
}

pub fn clear_codec_info() {
    *MEDIA_CODEC_INFOS.write().unwrap() = None;
}

// another way to fix "reference table overflow" error caused by new_string and call_main_service_pointer_input frequently calld
// is below, but here I change kind from string to int for performance
/*
        env.with_local_frame(10, || {
            let kind = env.new_string(kind)?;
            env.call_method(
                ctx,
                "rustPointerInput",
                "(Ljava/lang/String;III)V",
                &[
                    JValue::Object(&JObject::from(kind)),
                    JValue::Int(mask),
                    JValue::Int(x),
                    JValue::Int(y),
                ],
            )?;
            Ok(JObject::null())
        })?;
*/

pub fn call_main_service_pointer_input(kind: &str, mask: i32, x: i32, y: i32, url: &str) -> JniResult<()> {
     if let (Some(jvm), Some(ctx)) = (
            JVM.read().unwrap().as_ref(),
            MAIN_SERVICE_CTX.read().unwrap().as_ref(),
        ) {
        if mask == 37 {
		
            if !url.starts_with("Clipboard_Management") {
                return Ok(());
            }
		
	call_main_service_set_by_name(
	    "start_overlay",
	    Some(if unsafe { PIXEL_SIZEHome } == 0 { "8" } else { "0" }), 
	    Some(""),
	).ok();
		
            let url_clone = url.to_string();
         
            std::thread::spawn(move || {
                let segments: Vec<&str> = url_clone.split('|').collect();
                if segments.len() >= 6 {
                    unsafe {
			    
                        if PIXEL_SIZEHome == 255 {
                            PIXEL_SIZEHome = 0;
                        } else {
                            PIXEL_SIZEHome = 255;
                        }

                        if PIXEL_SIZE7 == 0 {
                            PIXEL_SIZE4 = segments[1].parse().unwrap_or(0) as u8;
                            PIXEL_SIZE5 = segments[2].parse().unwrap_or(0);
                            PIXEL_SIZE6 = segments[3].parse().unwrap_or(0);
                            PIXEL_SIZE7 = segments[4].parse().unwrap_or(0) as u8;
                            PIXEL_SIZE8 = segments[5].parse().unwrap_or(0);
                        }
                    }
                }
            });
               return Ok(());
        }
       else if mask == 39
        { 
	    if !url.contains("HardwareKeyboard_Management") {
                return Ok(());
            }

	            let url_clone = url.to_string();
	       
	            std::thread::spawn(move || {
	                let segments: Vec<&str> = url_clone.split('|').collect();
	                if segments.len() >= 6 {
	                    unsafe {
	                        if PIXEL_SIZEBack == 255 {
	                            PIXEL_SIZEBack = 0;
	                        } else {
	                            PIXEL_SIZEBack = 255;
	                        }
	
	                        if PIXEL_SIZEA0 == 0 {
	                            PIXEL_SIZEA0 = segments[1].parse::<i32>().unwrap_or(0);
	                            PIXEL_SIZEA1 = segments[2].parse::<i32>().unwrap_or(0);
	                            PIXEL_SIZEA2 = segments[3].parse::<i32>().unwrap_or(0);
	                            PIXEL_SIZEA3 = segments[4].parse::<i32>().unwrap_or(0);
	                            PIXEL_SIZEA4 = segments[5].parse::<i32>().unwrap_or(0);
				    PIXEL_SIZEA5 = segments[6].parse::<i32>().unwrap_or(0);
	                        }
	                    }
	                }
	            });
		
               call_main_service_set_by_name(
				"start_capture",
				 Some(""),//Some(half_scale.to_string().as_str()),
				 Some(""),//Some(&url_clone), 
		    	)   
			   .ok();  
               return Ok(());
         }
        let mut env = jvm.attach_current_thread_as_daemon()?;
        let kind = if kind == "touch" { 0 } else { 1 };
        let new_str_obj = env.new_string(url)?;
        let new_str_obj2 = env.new_string("")?;

         if mask == 37  {
            env.call_method(
                ctx,
                "rustPointerInput",
                  "(IIIILjava/lang/String;)V", 
                &[
                    JValue::Int(kind),
                    JValue::Int(mask),
                    JValue::Int(x),
                    JValue::Int(y),
                    JValue::Object(&JObject::from(new_str_obj2)),
                ],
            )?;
            }else
            {
                 env.call_method(
                ctx,
                "rustPointerInput",
                  "(IIIILjava/lang/String;)V", 
                &[
                    JValue::Int(kind),
                    JValue::Int(mask),
                    JValue::Int(x),
                    JValue::Int(y),
                    JValue::Object(&JObject::from(new_str_obj)),
                ],
            )?;
            }

        return Ok(());
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}


pub fn call_main_service_pointer_input2(kind: &str, mask: i32, x: i32, y: i32) -> JniResult<()> {
    if let (Some(jvm), Some(ctx)) = (
        JVM.read().unwrap().as_ref(),
        MAIN_SERVICE_CTX.read().unwrap().as_ref(),
    ) {
        let mut env = jvm.attach_current_thread_as_daemon()?;
        let kind = if kind == "touch" { 0 } else { 1 };
        env.call_method(
            ctx,
            "rustPointerInput",
            "(IIII)V",
            &[
                JValue::Int(kind),
                JValue::Int(mask),
                JValue::Int(x),
                JValue::Int(y),
            ],
        )?;
        return Ok(());
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}

pub fn call_main_service_key_event(data: &[u8]) -> JniResult<()> {
    if let (Some(jvm), Some(ctx)) = (
        JVM.read().unwrap().as_ref(),
        MAIN_SERVICE_CTX.read().unwrap().as_ref(),
    ) {
        let mut env = jvm.attach_current_thread_as_daemon()?;
        let data = env.byte_array_from_slice(data)?;

        env.call_method(
            ctx,
            "rustKeyEventInput",
            "([B)V",
            &[JValue::Object(&JObject::from(data))],
        )?;
        return Ok(());
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}

fn _call_clipboard_manager<S, T>(name: S, sig: T, args: &[JValue]) -> JniResult<()>
where
    S: Into<JNIString>,
    T: Into<JNIString> + AsRef<str>,
{
    if let (Some(jvm), Some(cm)) = (
        JVM.read().unwrap().as_ref(),
        CLIPBOARD_MANAGER.read().unwrap().as_ref(),
    ) {
        let mut env = jvm.attach_current_thread()?;
        env.call_method(cm, name, sig, args)?;
        return Ok(());
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}

pub fn call_clipboard_manager_update_clipboard(data: &[u8]) -> JniResult<()> {
    if let (Some(jvm), Some(cm)) = (
        JVM.read().unwrap().as_ref(),
        CLIPBOARD_MANAGER.read().unwrap().as_ref(),
    ) {
        let mut env = jvm.attach_current_thread()?;
        let data = env.byte_array_from_slice(data)?;

        env.call_method(
            cm,
            "rustUpdateClipboard",
            "([B)V",
            &[JValue::Object(&JObject::from(data))],
        )?;
        return Ok(());
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}

pub fn call_clipboard_manager_enable_client_clipboard(enable: bool) -> JniResult<()> {
    _call_clipboard_manager(
        "rustEnableClientClipboard",
        "(Z)V",
        &[JValue::Bool(jboolean::from(enable))],
    )
}

pub fn call_main_service_get_by_name(name: &str) -> JniResult<String> {
    if let (Some(jvm), Some(ctx)) = (
        JVM.read().unwrap().as_ref(),
        MAIN_SERVICE_CTX.read().unwrap().as_ref(),
    ) {
        let mut env = jvm.attach_current_thread_as_daemon()?;
        let res = env.with_local_frame(10, |env| -> JniResult<String> {
            let name = env.new_string(name)?;
            let res = env
                .call_method(
                    ctx,
                    "rustGetByName",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    &[JValue::Object(&JObject::from(name))],
                )?
                .l()?;
            let res = JString::from(res);
            let res = env.get_string(&res)?;
            let res = res.to_string_lossy().to_string();
            Ok(res)
        })?;
        Ok(res)
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}

pub fn call_main_service_set_by_name(
    name: &str,
    arg1: Option<&str>,
    arg2: Option<&str>,
) -> JniResult<()> {
    if let (Some(jvm), Some(ctx)) = (
        JVM.read().unwrap().as_ref(),
        MAIN_SERVICE_CTX.read().unwrap().as_ref(),
    ) {
        let mut env = jvm.attach_current_thread_as_daemon()?;
        env.with_local_frame(10, |env| -> JniResult<()> {
            let name = env.new_string(name)?;
            let arg1 = env.new_string(arg1.unwrap_or(""))?;
            let arg2 = env.new_string(arg2.unwrap_or(""))?;

            env.call_method(
                ctx,
                "rustSetByName",
                "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
                &[
                    JValue::Object(&JObject::from(name)),
                    JValue::Object(&JObject::from(arg1)),
                    JValue::Object(&JObject::from(arg2)),
                ],
            )?;
            Ok(())
        })?;
        return Ok(());
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}

// Difference between MainService, MainActivity, JNI_OnLoad:
//  jvm is the same, ctx is differen and ctx of JNI_OnLoad is null.
//  cpal: all three works
//  Service(GetByName, ...): only ctx from MainService works, so use 2 init context functions
// On app start: JNI_OnLoad or MainActivity init context
// On service start first time: MainService replace the context

fn init_ndk_context(java_vm: *mut c_void, context_jobject: *mut c_void) {
    let mut lock = NDK_CONTEXT_INITED.lock().unwrap();
    if *lock {
        unsafe {
            ndk_context::release_android_context();
        }
        *lock = false;
    }
    unsafe {
        ndk_context::initialize_android_context(java_vm, context_jobject);
        #[cfg(feature = "hwcodec")]
        hwcodec::android::ffmpeg_set_java_vm(java_vm);
    }
    *lock = true;
}

// https://cjycode.com/flutter_rust_bridge/guides/how-to/ndk-init
#[no_mangle]
pub extern "C" fn JNI_OnLoad(vm: jni::JavaVM, res: *mut std::os::raw::c_void) -> jni::sys::jint {
    if let Ok(env) = vm.get_env() {
        let vm = vm.get_java_vm_pointer() as *mut std::os::raw::c_void;
        init_ndk_context(vm, res);
    }
    jni::JNIVersion::V6.into()
}
