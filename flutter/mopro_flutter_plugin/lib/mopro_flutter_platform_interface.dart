import 'dart:typed_data';

import 'package:plugin_platform_interface/plugin_platform_interface.dart';

import 'mopro_flutter_method_channel.dart';

/// Represents the result of a zkEmail proving operation.
class ProveJwtResult {
  final Uint8List? proof;
  final String? error;

  ProveJwtResult({this.proof, this.error});

  /// Creates a result object from a map, typically returned by the method channel.
  factory ProveJwtResult.fromMap(Map<dynamic, dynamic> map) {
    return ProveJwtResult(
      proof:
          map['proof'] != null
              ? Uint8List.fromList(List<int>.from(map['proof']))
              : null,
      error: map['error'],
    );
  }
}

class VerifyJwtProofResult {
  final bool isValid;
  final String? error;

  VerifyJwtProofResult({required this.isValid, this.error});

  factory VerifyJwtProofResult.fromMap(Map<dynamic, dynamic> map) {
    return VerifyJwtProofResult(
      isValid: map['isValid'] ?? false,
      error: map['error'],
    );
  }
}

abstract class MoproFlutterPlatform extends PlatformInterface {
  /// Constructs a MoproFlutterPlatform.
  MoproFlutterPlatform() : super(token: _token);

  static final Object _token = Object();

  static MoproFlutterPlatform _instance = MethodChannelMoproFlutter();

  /// The default instance of [ZkemailFlutterPackagePlatform] to use.
  ///
  /// Defaults to [MethodChannelMoproFlutter].
  static MoproFlutterPlatform get instance => _instance;

  /// Platform-specific implementations should set this with their own
  /// platform-specific class that extends [MoproFlutterPlatform] when
  /// they register themselves.
  static set instance(MoproFlutterPlatform instance) {
    PlatformInterface.verifyToken(instance, _token);
    _instance = instance;
  }

  Future<String?> getPlatformVersion() {
    throw UnimplementedError('platformVersion() has not been implemented.');
  }

  Future<String> getApplicationDocumentsDirectory() {
    throw UnimplementedError(
      'getApplicationDocumentsDirectory() has not been implemented.',
    );
  }

  /// Generates a zkEmail proof.
  ///
  /// Takes the path to the Serialized Rekeying Set (SRS) file and the inputs map.
  /// The inputs map structure should match the one expected by the native mopro library,
  /// derived from the zkemail_input.json structure.
  Future<ProveJwtResult> proveJwt(
    String srsPath,
    String ephemeralPublicKey,
    String ephemeralSalt,
    String ephemeralExpiry,
    String tokenId,
    String jwt,
    String domain,
  ) {
    throw UnimplementedError('proveJwt() has not been implemented.');
  }

  Future<VerifyJwtProofResult> verifyJwtProof(
    String srsPath,
    Uint8List proof,
    String domain,
    String googleJwtPubkeyModulus,
    String ephemeralPubkey,
    String ephemeralPubkeyExpiry,
  ) {
    throw UnimplementedError('verifyJwtProof() has not been implemented.');
  }

  Future<String> signMessage(
    String anonGroupId,
    String text,
    bool internal,
    String ephemeralPublicKey,
    String ephemeralPrivateKey,
    String ephemeralPubkeyExpiry,
  ) {
    throw UnimplementedError('signMessage() has not been implemented.');
  }

  Future<String> generateEphemeralKey() {
    throw UnimplementedError(
      'generateEphemeralKey() has not been implemented.',
    );
  }
}
