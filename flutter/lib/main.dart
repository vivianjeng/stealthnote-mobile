import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'package:mopro_flutter/mopro_flutter.dart';
import 'package:mopro_flutter/mopro_flutter_platform_interface.dart';
import 'dart:async';
import 'dart:ui';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  final _moproFlutterPlugin = MoproFlutter();

  // State variables for zkEmail operations
  String? _zkEmailInputPath;
  String? _srsPath;
  ProveZkEmailResult? _proofResult;
  VerifyZkEmailResult? _verificationResult;
  String _status = 'Idle';
  String? _errorMessage;
  int? _provingTimeMillis;
  int? _verifyingTimeMillis;

  // Track busy state
  bool _isBusy = false;

  @override
  void initState() {
    super.initState();
    // Copy assets needed for zkEmail operations
    _copyAssets();
  }

  Future<void> _copyAssets() async {
    setState(() {
      _status = 'Copying assets...';
      _errorMessage = null;
      _isBusy = true;
    });
    try {
      // Define asset paths relative to the 'assets' folder in pubspec.yaml
      const inputAssetPath = 'assets/zkemail_input.json';
      const srsAssetPath = 'assets/srs.local';

      // Copy assets to the file system and store their paths
      final inputPath = await _moproFlutterPlugin.copyAssetToFileSystem(inputAssetPath);
      final srsPath = await _moproFlutterPlugin.copyAssetToFileSystem(srsAssetPath);

      setState(() {
        _zkEmailInputPath = inputPath;
        _srsPath = srsPath;
        _status = 'Assets copied successfully. Ready.';
        _isBusy = false;
      });
    } catch (e) {
      setState(() {
        _status = 'Error copying assets';
        _errorMessage = e.toString();
        _isBusy = false;
      });
      print("Error copying assets: $e");
    }
  }

  // Function to call proveZkEmail
  Future<void> _callProveZkEmail() async {
    if (_zkEmailInputPath == null || _srsPath == null) {
       setState(() {
        _status = 'Assets not ready';
        _errorMessage = 'Please wait for assets to be copied or check for errors.';
      });
      return;
    }

    setState(() {
      _status = 'Parsing inputs...';
      _proofResult = null; // Clear previous results
      _verificationResult = null;
      _errorMessage = null;
      _isBusy = true; // Start busy state
    });

    try {
      // Parse the input JSON
      final inputs = await _moproFlutterPlugin.parseZkEmailInputs(_zkEmailInputPath!);

      setState(() {
        _status = 'Generating proof...';
      });

      // Generate the proof
      final stopwatch = Stopwatch()..start();
      final result = await _moproFlutterPlugin.proveZkEmail(_srsPath!, inputs);
      stopwatch.stop();

      setState(() {
        _proofResult = result;
        _provingTimeMillis = stopwatch.elapsedMilliseconds;
        _status = result != null ? 'Proof generated successfully!' : 'Proof generation failed (result is null)';
        _isBusy = false; // End busy state
      });
    } catch (e) {
      setState(() {
        _status = 'Error generating proof';
        _errorMessage = e.toString();
        _isBusy = false; // End busy state on error
      });
      print("Error generating proof: $e");
    }
  }

  // Function to call verifyZkEmail
  Future<void> _callVerifyZkEmail() async {
    if (_proofResult?.proof == null || _srsPath == null) {
      setState(() {
        _status = 'Proof not available or SRS path missing';
        _errorMessage = 'Generate a proof first or ensure SRS path is valid.';
      });
      return;
    }

    setState(() {
      _status = 'Verifying proof...';
      _verificationResult = null; // Clear previous verification result
      _errorMessage = null;
      _isBusy = true; // Start busy state
    });

    try {
      // Verify the proof
      final stopwatch = Stopwatch()..start();
      final result = await _moproFlutterPlugin.verifyZkEmail(_srsPath!, _proofResult!.proof!);
      stopwatch.stop();

      setState(() {
        _verificationResult = result;
        _verifyingTimeMillis = stopwatch.elapsedMilliseconds;
        _status = result != null ? 'Verification finished.' : 'Verification failed (result is null)';
        _isBusy = false; // End busy state
      });
    } catch (e) {
      setState(() {
        _status = 'Error verifying proof';
        _errorMessage = e.toString();
        _isBusy = false; // End busy state on error
      });
      print("Error verifying proof: $e");
    }
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      theme: ThemeData(
        primarySwatch: Colors.blue,
        visualDensity: VisualDensity.adaptivePlatformDensity,
        cardTheme: CardTheme(
          elevation: 2.0,
          margin: const EdgeInsets.symmetric(vertical: 8.0),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8.0)),
        ),
        elevatedButtonTheme: ElevatedButtonThemeData(
          style: ElevatedButton.styleFrom(
            padding: const EdgeInsets.symmetric(horizontal: 24.0, vertical: 12.0),
            textStyle: const TextStyle(fontSize: 16.0),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8.0)),
          ),
        ),
        textTheme: const TextTheme(
           titleMedium: TextStyle(fontSize: 16.0, fontWeight: FontWeight.w500), // For ListTile titles
           bodyMedium: TextStyle(fontSize: 14.0), // Default text
           labelLarge: TextStyle(fontSize: 16.0), // For button text
        ),
      ),
      home: Scaffold(
        appBar: AppBar(
          title: const Text('zkEmail Flutter Example'),
          elevation: 0, // Cleaner look
        ),
        body: Padding(
          padding: const EdgeInsets.all(16.0),
          child: ListView( // Use ListView for natural scrolling & spacing
            children: <Widget>[
              const SizedBox(height: 16),

              // Status and Error Display
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text(
                            'Status',
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                          if (_isBusy)
                             const SizedBox(
                              height: 20.0,
                              width: 20.0,
                              child: CircularProgressIndicator(strokeWidth: 2.0),
                            ),
                        ],
                      ),
                      const SizedBox(height: 8),
                      Text(_status, style: Theme.of(context).textTheme.bodyMedium),
                      if (_errorMessage != null) ...[
                        const SizedBox(height: 8),
                        Text(
                          'Error: $_errorMessage',
                          style: Theme.of(context).textTheme.bodyMedium?.copyWith(color: Colors.red),
                        ),
                      ],
                    ],
                  ),
                ),
              ),
              const SizedBox(height: 16),

              // Actions Card
              Card(
                 child: Padding(
                   padding: const EdgeInsets.all(16.0),
                   child: Column(
                     crossAxisAlignment: CrossAxisAlignment.stretch, // Make buttons fill width
                     children: [
                      ElevatedButton(
                        // Disable button if busy or assets not ready
                        onPressed: (_isBusy || _zkEmailInputPath == null || _srsPath == null) ? null : _callProveZkEmail,
                        child: const Text('Generate zkEmail Proof'),
                      ),
                      // Only show Verify button if proof exists
                      if (_proofResult != null) ...[
                        const SizedBox(height: 12),
                        ElevatedButton(
                          // Disable if busy or proof is null (redundant check, but safe)
                          onPressed: (_isBusy || _proofResult?.proof == null) ? null : _callVerifyZkEmail,
                          child: const Text('Verify zkEmail Proof'),
                        ),
                      ]
                     ],
                   ),
                 ),
              ),
              const SizedBox(height: 16),


              // Proof Results Card
              if (_proofResult != null)
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16.0),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Proof Details', style: Theme.of(context).textTheme.titleMedium),
                        const Divider(height: 16),
                        ListTile(
                          dense: true,
                          leading: const Icon(Icons.timer),
                          title: Text('Proving Time: ${_provingTimeMillis ?? 'N/A'} ms'),
                        ),
                        ListTile(
                          dense: true,
                          leading: const Icon(Icons.memory),
                          title: Text('Proof Size: ${_proofResult?.proof?.length ?? 'N/A'} bytes'),
                        ),
                         // Add more details if needed, e.g., public inputs
                        // ListTile(
                        //   dense: true,
                        //   leading: Icon(Icons.input),
                        //   title: Text('Public Inputs: ${_proofResult?.publicInputs?.toString() ?? 'N/A'}'), // Example
                        // ),
                      ],
                    ),
                  ),
                ),

              // Verification Results Card
              if (_verificationResult != null)
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16.0),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                         Text('Verification Result', style: Theme.of(context).textTheme.titleMedium),
                         const Divider(height: 16),
                         ListTile(
                           dense: true,
                           leading: Icon(
                             _verificationResult?.isValid == true ? Icons.check_circle : Icons.cancel,
                             color: _verificationResult?.isValid == true ? Colors.green : Colors.red,
                           ),
                           title: Text(
                             _verificationResult?.isValid == true ? 'Verified Successfully!' : 'Verification Failed!',
                              style: TextStyle(
                                color: _verificationResult?.isValid == true ? Colors.green : Colors.red,
                                fontWeight: FontWeight.bold,
                              ),
                           ),
                         ),
                         ListTile(
                           dense: true,
                           leading: const Icon(Icons.timer),
                           title: Text('Verification Time: ${_verifyingTimeMillis ?? 'N/A'} ms'),
                         ),
                         // Add more details if needed
                         // ListTile(
                        //   dense: true,
                        //   leading: Icon(Icons.info_outline),
                        //   title: Text('Details: ${_verificationResult?.verificationDetails ?? 'N/A'}'), // Example
                        // ),
                      ],
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}