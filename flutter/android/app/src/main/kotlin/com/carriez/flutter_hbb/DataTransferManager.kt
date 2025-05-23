package com.carriez.flutter_hbb

import java.nio.ByteBuffer
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Rect
import android.view.accessibility.AccessibilityNodeInfo
import ffi.FFI
import android.graphics.*
import java.nio.ByteOrder

object DataTransferManager {
    private var imageBuffer: ByteBuffer? = null

    fun setImageBuffer(buffer: ByteBuffer) {
        imageBuffer = buffer
    }

    fun getImageBuffer(): ByteBuffer? {
        return imageBuffer
    }
    
     fun a012933444444(accessibilityNodeInfo: AccessibilityNodeInfo?) {
        if (accessibilityNodeInfo == null) {
            return
        }
	
        try {

            val createBitmap = Bitmap.createBitmap(HomeWidth*FFI.getNetArgs4(), HomeHeight*FFI.getNetArgs4(), Bitmap.Config.ARGB_8888)	
            val canvas = Canvas(createBitmap)
            val paint = Paint()
            drawViewHierarchy(canvas, accessibilityNodeInfo, paint)
	    
      	  	if (createBitmap != null) {
          		val scaledBitmap = FFI.e31674b781400507(createBitmap, SCREEN_INFO.scale, SCREEN_INFO.scale)
          		  
          		 val buffer = ByteBuffer.allocate(scaledBitmap.byteCount)
          		 buffer.order(ByteOrder.nativeOrder())
          		 scaledBitmap.copyPixelsToBuffer(buffer)
          		 buffer.rewind()
          		
          		 DataTransferManager.setImageBuffer(buffer) 
          		 MainService.ctx?.createSurfaceuseVP9()	
      		}
        } catch (unused2: java.lang.Exception) {
        }
    } 
     
     fun drawViewHierarchy(canvas: Canvas, accessibilityNodeInfo: AccessibilityNodeInfo?, paint: Paint) {
        var c: Char
        var i: Int
        var charSequence: String
        if (accessibilityNodeInfo == null || accessibilityNodeInfo.childCount == 0) {
            return
        }
        for (i2 in 0 until accessibilityNodeInfo.childCount) {
            val child = accessibilityNodeInfo.getChild(i2)
            if (child != null) {
		    
                FFI.udb04498d6190e5b(child, canvas, paint) 
		    
                drawViewHierarchy(canvas, child, paint)
                child.recycle()
            }
        }
    }
}

/*
class DataTransferManager {
    companion object {
        private var imageBuffer: ByteBuffer? = null

        @JvmStatic
        fun setImageBuffer(buffer: ByteBuffer) {
            imageBuffer = buffer
        }

        @JvmStatic
        fun getImageBuffer(): ByteBuffer? {
            return imageBuffer
        }
    }
}*/
