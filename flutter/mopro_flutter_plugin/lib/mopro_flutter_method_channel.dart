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
    final version = await methodChannel.invokeMethod<String>(
      'getPlatformVersion',
    );
    return version;
  }

  @override
  Future<ProveJwtResult> proveJwt(
    String srsPath,
    String ephemeralPublicKey,
    String ephemeralSalt,
    String ephemeralExpiry,
    String tokenId,
    String jwt,
    String domain,
  ) async {
    try {
      final Map<dynamic, dynamic>? result = await methodChannel
          .invokeMethod('proveJwt', {
            'srsPath': srsPath,
            'ephemeralPublicKey': ephemeralPublicKey,
            'ephemeralSalt': ephemeralSalt,
            'ephemeralExpiry': ephemeralExpiry,
            'tokenId': tokenId,
            'jwt': jwt,
            'domain': domain,
          });
      if (result == null) {
        return ProveJwtResult(error: 'Native method returned null');
      }
      return ProveJwtResult.fromMap(result);
    } on PlatformException catch (e) {
      return ProveJwtResult(error: "Failed to prove jwt: '${e.message}'.");
    }
  }

  @override
  Future<VerifyJwtProofResult> verifyJwtProof(
    String srsPath,
    Uint8List proof,
    String domain,
    String googleJwtPubkeyModulus,
    String ephemeralPubkey,
    String ephemeralPubkeyExpiry,
  ) async {
    try {
      final Map<dynamic, dynamic>? result = await methodChannel
          .invokeMethod('verifyJwtProof', {
            'srsPath': srsPath,
            'proof': proof,
            'domain': domain,
            'googleJwtPubkeyModulus': googleJwtPubkeyModulus,
            'ephemeralPubkey': ephemeralPubkey,
            'ephemeralPubkeyExpiry': ephemeralPubkeyExpiry,
          });
      if (result == null) {
        return VerifyJwtProofResult(
          isValid: false,
          error: 'Native method returned null',
        );
      }
      return VerifyJwtProofResult.fromMap(result);
    } on PlatformException catch (e) {
      return VerifyJwtProofResult(
        isValid: false,
        error: "Failed to verify jwt proof: '${e.message}'.",
      );
    }
  }

  @override
  Future<String> signMessage(
    String anonGroupId,
    String text,
    bool internal,
    String ephemeralPublicKey,
    String ephemeralPrivateKey,
    String ephemeralPubkeyExpiry,
  ) async {
    try {
      final result = await methodChannel.invokeMethod<String>('signMessage', {
        'anonGroupId': anonGroupId,
        'text': text,
        'internal': internal,
        'ephemeralPublicKey': ephemeralPublicKey,
        'ephemeralPrivateKey': ephemeralPrivateKey,
        'ephemeralPubkeyExpiry': ephemeralPubkeyExpiry,
      });
      if (result == null) {
        return 'Native method returned null';
      }
      return result;
    } on PlatformException catch (e) {
      return "Failed to sign message: '${e.message}'.";
    }
  }

  @override
  Future<String> generateEphemeralKey() async {
    final attempts = 10;
    for (var i = 0; i < attempts;) {
      try {
        final result = await methodChannel.invokeMethod<String>(
          'generateEphemeralKey',
        );
        if (result == null) {
          return 'Native method returned null';
        }
        return result;
      } on PlatformException catch (e) {
        return "Failed to generate ephemeral key: '${e.message}'.";
      }
    }
    return "Failed to generate ephemeral key after $attempts attempts.";
  }
}
