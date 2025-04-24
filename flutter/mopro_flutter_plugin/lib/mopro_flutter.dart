import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/services.dart' show rootBundle;
import 'package:path_provider/path_provider.dart';
import 'mopro_flutter_platform_interface.dart';

class MoproFlutter {
  Future<String?> getPlatformVersion() {
    return MoproFlutterPlatform.instance.getPlatformVersion();
  }

  /// Copies a file from the app's assets to the device's file system.
  ///
  /// This is useful for making asset files like SRS or input JSONs available
  /// to the native code.
  Future<String> copyAssetToFileSystem(String assetPath) async {
    // Load the asset as bytes
    final byteData = await rootBundle.load(assetPath);

    // Get the app's document directory (or other accessible directory)
    final directory = await getApplicationDocumentsDirectory();

    // Strip off the initial dirs from the filename if present
    final filename = assetPath.split('/').last;

    final file = File('${directory.path}/$filename');

    // Write the bytes to a file in the file system
    await file.writeAsBytes(byteData.buffer.asUint8List());

    return file.path; // Return the file path
  }

  /// Parses a zkEmail input JSON file from the file system.
  ///
  /// Reads the JSON file specified by [filePath], parses it,
  /// and returns a Map structured for the `proveZkEmail` method.
  Future<Map<String, List<String>>> parseZkEmailInputs(String filePath) async {
    try {
      // Read the file from the provided file system path
      final file = File(filePath);
      final jsonString = await file.readAsString();
      final jsonObject = jsonDecode(jsonString) as Map<String, dynamic>;

      final inputs = <String, List<String>>{};

      // Helper function to convert JSON array elements to String list
      List<String> jsonArrayToStringList(List<dynamic> jsonArray) {
        return jsonArray.map((item) => item.toString()).toList();
      }

      // Extract data based on the structure observed in native examples
      if (jsonObject.containsKey('header')) {
        final header = jsonObject['header'] as Map<String, dynamic>;
        if (header.containsKey('storage') && header['storage'] is List) {
          inputs['header_storage'] = jsonArrayToStringList(header['storage'] as List);
        }
        if (header.containsKey('len')) {
          inputs['header_len'] = [header['len'].toString()];
        }
      }

      if (jsonObject.containsKey('pubkey')) {
        final pubkey = jsonObject['pubkey'] as Map<String, dynamic>;
        if (pubkey.containsKey('modulus') && pubkey['modulus'] is List) {
          inputs['pubkey_modulus'] = jsonArrayToStringList(pubkey['modulus'] as List);
        }
        if (pubkey.containsKey('redc') && pubkey['redc'] is List) {
          inputs['pubkey_redc'] = jsonArrayToStringList(pubkey['redc'] as List);
        }
      }

      if (jsonObject.containsKey('signature') && jsonObject['signature'] is List) {
        inputs['signature'] = jsonArrayToStringList(jsonObject['signature'] as List);
      }

      if (jsonObject.containsKey('date_index')) {
        inputs['date_index'] = [jsonObject['date_index'].toString()];
      }

      void extractSequence(String key, String mapKey) {
          if (jsonObject.containsKey(key)) {
              final sequence = jsonObject[key] as Map<String, dynamic>;
              if (sequence.containsKey('index')) {
                  inputs['${mapKey}_index'] = [sequence['index'].toString()];
              }
              if (sequence.containsKey('length')) {
                  inputs['${mapKey}_length'] = [sequence['length'].toString()];
              }
          }
      }

      extractSequence('subject_sequence', 'subject');
      extractSequence('from_header_sequence', 'from_header');
      extractSequence('from_address_sequence', 'from_address');

      return inputs;
    } catch (e) {
      print("Error parsing zkEmail inputs: $e");
      rethrow; // Re-throw the exception so the caller can handle it
    }
  }


  /// Generates a zkEmail proof using the platform channel.
  ///
  /// Takes the path to the SRS file (must be accessible by native code,
  /// use [copyAssetToFileSystem] if needed) and the parsed inputs map
  /// (use [parseZkEmailInputs] to generate from JSON).
  Future<ProveZkEmailResult?> proveZkEmail(
    String srsPath,
    Map<String, List<String>> inputs
    ) {
      return MoproFlutterPlatform.instance.proveZkEmail(srsPath, inputs);
  }

  /// Verifies a zkEmail proof using the platform channel.
  ///
  /// Takes the path to the SRS file (must be accessible by native code,
  /// use [copyAssetToFileSystem] if needed) and the proof bytes.
  Future<VerifyZkEmailResult?> verifyZkEmail(
    String srsPath,
    Uint8List proof
  ) {
    return MoproFlutterPlatform.instance.verifyZkEmail(srsPath, proof);
  }
}