import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import 'mopro_flutter_platform_interface.dart';

/// An implementation of [MoproFlutterPlatform] that uses method channels.
class MethodChannelMoproFlutter extends MoproFlutterPlatform {
  /// The method channel used to interact with the native platform.
  @visibleForTesting
  final methodChannel = const MethodChannel('mopro_flutter');

  @override
  Future<String?> getPlatformVersion() async {
    final version = await methodChannel.invokeMethod<String>('getPlatformVersion');
    return version;
  }

  @override
  Future<ProveJwtResult> proveJwt(String srsPath, Map<String, List<String>> inputs) async {
    try {
      final Map<dynamic, dynamic>? result = await methodChannel.invokeMethod(
        'proveJwt',
        {
          'srsPath': srsPath,
          'inputs': inputs,
        },
      );
      if (result == null) {
        return ProveJwtResult(error: 'Native method returned null');
      }
      return ProveJwtResult.fromMap(result);
    } on PlatformException catch (e) {
      return ProveJwtResult(error: "Failed to prove jwt: '${e.message}'.");
    }
  }

  @override
  Future<VerifyJwtResult> verifyJwt(String srsPath, Uint8List proof) async {
    try {
      final Map<dynamic, dynamic>? result = await methodChannel.invokeMethod(
        'verifyJwt',
        {
          'srsPath': srsPath,
          'proof': proof,
        },
      );
      if (result == null) {
        return VerifyJwtResult(isValid: false, error: 'Native method returned null');
      }
      return VerifyJwtResult.fromMap(result);
    } on PlatformException catch (e) {
      return VerifyJwtResult(isValid: false, error: "Failed to verify jwt: '${e.message}'.");
    }
  }
}