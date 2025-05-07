import 'dart:convert';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:mopro_flutter/mopro_flutter.dart';
import 'package:mopro_flutter/mopro_flutter_platform_interface.dart';

Future<String> getEphemeralKey() async {
  const secureStorage = FlutterSecureStorage();
  final moproFlutterPlugin = MoproFlutter();
  // await secureStorage.delete(key: 'ephemeral_key');
  String? ephemeralKey = await secureStorage.read(key: 'ephemeral_key');

  if (ephemeralKey == null) {
    try {
      // Generate new ephemeral key
      ephemeralKey = await moproFlutterPlugin.generateEphemeralKey();
    } catch (e) {
      print('Error generating ephemeral key: $e');
      throw e;
    }
    // Save it securely
    await secureStorage.write(key: 'ephemeral_key', value: ephemeralKey);
  } else {
    // check if the ephemeral key is expired
    Map<String, dynamic> ephemeralKeyObj = jsonDecode(ephemeralKey);
    final ephemeralExpiry = ephemeralKeyObj['expiry'];

    if (DateTime.parse(ephemeralExpiry).isBefore(DateTime.now())) {
      // delete the ephemeral key
      await secureStorage.delete(key: 'ephemeral_key');
      try {
        // Generate new ephemeral key
        ephemeralKey = await moproFlutterPlugin.generateEphemeralKey();
      } catch (e) {
        print('Error generating ephemeral key: $e');
        throw e;
      }
      // Save it securely
      await secureStorage.write(key: 'ephemeral_key', value: ephemeralKey);
    } else {
      print('not expired');
    }
    print('Found existing ephemeralKey: $ephemeralKey');
  }

  // At this point, `ephemeralKey` is guaranteed to be non-null
  // You can use it safely
  return ephemeralKey;
}

Future<void> deleteEphemeralKey() async {
  const secureStorage = FlutterSecureStorage();
  await secureStorage.delete(key: 'ephemeral_key');
}
