import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:mopro_flutter/mopro_flutter.dart';
import 'package:mopro_flutter/mopro_flutter_platform_interface.dart';

Future<String> getEphemeralKey() async {
  final secureStorage = FlutterSecureStorage();
  final moproFlutterPlugin = MoproFlutter();
  // await secureStorage.delete(key: 'ephemeral_key');
  String? ephemeralKey = await secureStorage.read(key: 'ephemeral_key');

  if (ephemeralKey == null) {
    // Generate new ephemeral key
    ephemeralKey = await moproFlutterPlugin.generateEphemeralKey();

    // Save it securely
    await secureStorage.write(key: 'ephemeral_key', value: ephemeralKey);
  } else {
    // TODO: check if the ephemeral key is expired
    print('Found existing ephemeralKey: $ephemeralKey');
  }

  // At this point, `ephemeralKey` is guaranteed to be non-null
  // You can use it safely
  return ephemeralKey;
}
