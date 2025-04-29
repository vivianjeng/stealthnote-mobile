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

  /// Parses a Jwt input JSON file from the file system.
  ///
  /// Reads the JSON file specified by [filePath], parses it,
  /// and returns a Map structured for the `proveJwt` method.
  Future<Map<String, List<String>>> parseJwtInputs(String filePath) async {
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
      if (jsonObject.containsKey('partial_data')) {
        final partialData = jsonObject['partial_data'] as Map<String, dynamic>;
        if (partialData.containsKey('storage') && partialData['storage'] is List) {
          inputs['partial_data_storage'] = jsonArrayToStringList(partialData['storage'] as List);
        }
        if (partialData.containsKey('len')) {
          inputs['partial_data_len'] = [partialData['len'].toString()];
        }
      }

      if (jsonObject.containsKey('partial_hash') && jsonObject['partial_hash'] is List) {
        inputs['partial_hash'] = jsonArrayToStringList(jsonObject['partial_hash'] as List);
      }

      if (jsonObject.containsKey('full_data_length')) {
        inputs['full_data_length'] = [jsonObject['full_data_length'].toString()];
      }

      if (jsonObject.containsKey('base64_decode_offset')) {
        inputs['base64_decode_offset'] = [jsonObject['base64_decode_offset'].toString()];
      }

      if (jsonObject.containsKey('jwt_pubkey_modulus_limbs') && jsonObject['jwt_pubkey_modulus_limbs'] is List) {
        inputs['jwt_pubkey_modulus_limbs'] = jsonArrayToStringList(jsonObject['jwt_pubkey_modulus_limbs'] as List);
      }

      if (jsonObject.containsKey('jwt_pubkey_redc_params_limbs') && jsonObject['jwt_pubkey_redc_params_limbs'] is List) {
        inputs['jwt_pubkey_redc_params_limbs'] = jsonArrayToStringList(jsonObject['jwt_pubkey_redc_params_limbs'] as List);
      }

      if (jsonObject.containsKey('jwt_signature_limbs') && jsonObject['jwt_signature_limbs'] is List) {
        inputs['jwt_signature_limbs'] = jsonArrayToStringList(jsonObject['jwt_signature_limbs'] as List);
      }

       if (jsonObject.containsKey('domain')) {
        final domainData = jsonObject['domain'] as Map<String, dynamic>;
        if (domainData.containsKey('storage') && domainData['storage'] is List) {
          inputs['domain_storage'] = jsonArrayToStringList(domainData['storage'] as List);
        }
        if (domainData.containsKey('len')) {
          inputs['domain_len'] = [domainData['len'].toString()];
        }
      }

      if (jsonObject.containsKey('ephemeral_pubkey')) {
        inputs['ephemeral_pubkey'] = [jsonObject['ephemeral_pubkey'].toString()];
      }

      if (jsonObject.containsKey('ephemeral_pubkey_salt')) {
        inputs['ephemeral_pubkey_salt'] = [jsonObject['ephemeral_pubkey_salt'].toString()];
      }

      if (jsonObject.containsKey('ephemeral_pubkey_expiry')) {
        inputs['ephemeral_pubkey_expiry'] = [jsonObject['ephemeral_pubkey_expiry'].toString()];
      }
      return inputs;
    } catch (e) {
      print("Error parsing jwt inputs: $e");
      rethrow; // Re-throw the exception so the caller can handle it
    }
  }


  /// Generates a Jwt proof using the platform channel.
  ///
  /// Takes the path to the SRS file (must be accessible by native code,
  /// use [copyAssetToFileSystem] if needed) and the parsed inputs map
  /// (use [parseJwtInputs] to generate from JSON).
  Future<ProveJwtResult?> proveJwt(
    String srsPath,
    Map<String, List<String>> inputs
    ) {
      return MoproFlutterPlatform.instance.proveJwt(srsPath, inputs);
  }

  /// Verifies a Jwt proof using the platform channel.
  ///
  /// Takes the path to the SRS file (must be accessible by native code,
  /// use [copyAssetToFileSystem] if needed) and the proof bytes.
  Future<VerifyJwtResult?> verifyJwt(
    String srsPath,
    Uint8List proof
  ) {
    return MoproFlutterPlatform.instance.verifyJwt(srsPath, proof);
  }
}