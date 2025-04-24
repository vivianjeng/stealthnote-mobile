import 'dart:typed_data';

import 'package:plugin_platform_interface/plugin_platform_interface.dart';

import 'mopro_flutter_method_channel.dart';

/// Represents the result of a zkEmail proving operation.
class ProveZkEmailResult {
  final Uint8List? proof;
  final String? error;

  ProveZkEmailResult({this.proof, this.error});

  /// Creates a result object from a map, typically returned by the method channel.
  factory ProveZkEmailResult.fromMap(Map<dynamic, dynamic> map) {
    return ProveZkEmailResult(
      proof: map['proof'] != null ? Uint8List.fromList(List<int>.from(map['proof'])) : null,
      error: map['error'],
    );
  }
}

/// Represents the result of a zkEmail verification operation.
class VerifyZkEmailResult {
  final bool isValid;
  final String? error;

  VerifyZkEmailResult({required this.isValid, this.error});

    /// Creates a result object from a map, typically returned by the method channel.
  factory VerifyZkEmailResult.fromMap(Map<dynamic, dynamic> map) {
    return VerifyZkEmailResult(
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
    throw UnimplementedError('getApplicationDocumentsDirectory() has not been implemented.');
  }

  /// Generates a zkEmail proof.
  ///
  /// Takes the path to the Serialized Rekeying Set (SRS) file and the inputs map.
  /// The inputs map structure should match the one expected by the native mopro library,
  /// derived from the zkemail_input.json structure.
  Future<ProveZkEmailResult> proveZkEmail(String srsPath, Map<String, List<String>> inputs) {
    throw UnimplementedError('proveZkEmail() has not been implemented.');
  }

  /// Verifies a zkEmail proof.
  ///
  /// Takes the path to the SRS file and the proof bytes.
  Future<VerifyZkEmailResult> verifyZkEmail(String srsPath, Uint8List proof) {
    throw UnimplementedError('verifyZkEmail() has not been implemented.');
  }
}