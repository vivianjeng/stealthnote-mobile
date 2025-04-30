package com.example.mopro_flutter

import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import io.flutter.plugin.common.MethodChannel.MethodCallHandler
import io.flutter.plugin.common.MethodChannel.Result

import io.flutter.plugin.common.StandardMethodCodec
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

import uniffi.mopro.proveJwt
import uniffi.mopro.verifyJwt
import uniffi.mopro.verifyJwtProof

/** MoproFlutterPlugin */
class MoproFlutterPlugin : FlutterPlugin, MethodCallHandler {
    /// The MethodChannel that will the communication between Flutter and native Android
    ///
    /// This local reference serves to register the plugin with the Flutter Engine and unregister it
    /// when the Flutter Engine is detached from the Activity
    private lateinit var channel: MethodChannel
    private val scope = CoroutineScope(Dispatchers.IO)

    override fun onAttachedToEngine(flutterPluginBinding: FlutterPlugin.FlutterPluginBinding) {
        channel = MethodChannel(
            flutterPluginBinding.binaryMessenger,
            "mopro_flutter",
            StandardMethodCodec.INSTANCE
        )
        channel.setMethodCallHandler(this)
    }

    override fun onMethodCall(call: MethodCall, result: Result) {
        scope.launch {
          try {
            when (call.method) {
              "getPlatformVersion" -> {
                result.success("Android ${android.os.Build.VERSION.RELEASE}")
              }
              "proveJwt" -> {
                val srsPath = call.argument<String>("srsPath")
                @Suppress("UNCHECKED_CAST")
                val inputs = call.argument<Map<String, List<String>>>("inputs")
    
                if (srsPath == null || inputs == null) {
                  launch(Dispatchers.Main) {
                    result.error("INVALID_ARGUMENTS", "srsPath or inputs is null", null)
                  }
                  return@launch
                }
    
                val proofBytes = proveJwt(srsPath!!, inputs!!)
                launch(Dispatchers.Main) {
                  result.success(mapOf("proof" to proofBytes, "error" to null))
                }
              }
              "verifyJwt" -> {
                val srsPath = call.argument<String>("srsPath")
                val proof = call.argument<ByteArray>("proof")
    
                if (srsPath == null || proof == null) {
                  launch(Dispatchers.Main) {
                    result.error("INVALID_ARGUMENTS", "srsPath or proof is null", null)
                  }
                  return@launch
                }
    
                val isValid = verifyJwt(srsPath!!, proof!!)
                launch(Dispatchers.Main) {
                  result.success(mapOf("isValid" to isValid, "error" to null))
                }
              }
              "verifyJwtProof" -> {
                val srsPath = call.argument<String>("srsPath")
                val proof = call.argument<ByteArray>("proof")
                val domain = call.argument<String>("domain")
                val googleJwtPubkeyModulus = call.argument<String>("googleJwtPubkeyModulus")
                val ephemeralPubkey = call.argument<String>("ephemeralPubkey")
                val ephemeralPubkeyExpiry = call.argument<String>("ephemeralPubkeyExpiry")

                val isValid = verifyJwtProof(srsPath!!, proof!!, domain!!, googleJwtPubkeyModulus!!, ephemeralPubkey!!, ephemeralPubkeyExpiry!!)
                launch(Dispatchers.Main) {
                  result.success(mapOf("isValid" to isValid, "error" to null))
                }
              }
              else -> {
                launch(Dispatchers.Main) {
                  result.notImplemented()
                }
              }
            }
          } catch (e: Exception) {
            launch(Dispatchers.Main) {
              when (call.method) {
                "proveJwt" -> result.success(mapOf("proof" to null, "error" to e.message))
                "verifyJwt" -> result.success(mapOf("isValid" to false, "error" to e.message))
                else -> result.error("NATIVE_ERROR", e.message, e.stackTraceToString())
              }
            }
          }
        }
    }
    
    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        channel.setMethodCallHandler(null)
    }
}
