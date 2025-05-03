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
import uniffi.mopro.verifyJwtProof
import uniffi.mopro.EphemeralKey
import uniffi.mopro.signMessage

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
                val ephemeralPublicKey = call.argument<String>("ephemeralPublicKey")
                val ephemeralSalt = call.argument<String>("ephemeralSalt")
                val ephemeralExpiry = call.argument<String>("ephemeralExpiry")
                val tokenId = call.argument<String>("tokenId")
                val jwt = call.argument<String>("jwt")
                val domain = call.argument<String>("domain")
    
                if (srsPath == null || inputs == null) {
                  launch(Dispatchers.Main) {
                    result.error("INVALID_ARGUMENTS", "srsPath or inputs is null", null)
                  }
                  return@launch
                }
    
                val proofBytes = proveJwt(srsPath!!, ephemeralPublicKey!!, ephemeralSalt!!, ephemeralExpiry!!, tokenId!!, jwt!!, domain!!)
                launch(Dispatchers.Main) {
                  result.success(mapOf("proof" to proofBytes, "error" to null))
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
              "signMessage" -> {
                val anonGroupId = call.argument<String>("anonGroupId")
                val text = call.argument<String>("text")
                val internal = call.argument<Boolean>("internal")
                  val ephemeralPublicKey = call.argument<String>("ephemeralPublicKey")
                  val ephemeralPrivateKey = call.argument<String>("ephemeralPrivateKey")
                val ephemeralPubkeyExpiry = call.argument<String>("ephemeralPubkeyExpiry")

                if (anonGroupId == null || text == null || internal == null || ephemeralPublicKey == null || ephemeralPrivateKey == null || ephemeralPubkeyExpiry == null) {
                  launch(Dispatchers.Main) {
                    result.error("INVALID_ARGUMENTS", "anonGroupId or text or internal or ephemeralPublicKey or ephemeralPrivateKey or ephemeralPubkeyExpiry is null", null)
                  }
                  return@launch
                }

                val signedMessage = signMessage(anonGroupId!!, text!!, internal!!, ephemeralPublicKey!!, ephemeralPrivateKey!!, ephemeralPubkeyExpiry!!)
                launch(Dispatchers.Main) {
                  result.success(signedMessage)
                }
              }
              "generateEphemeralKey" -> {
                val ephemeralKey = generateEphemeralKey()
                launch(Dispatchers.Main) {
                  result.success(ephemeralKey)
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
                "signMessage" -> result.success(mapOf("signedMessage" to null, "error" to e.message))
                "verifyJwtProof" -> result.success(mapOf("isValid" to false, "error" to e.message))
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
