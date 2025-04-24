# Cross-Platform Mobile ZKPs

Flutter is a popular cross-platform mobile app development framework. Mopro Flutter shows an example of integrating ZK-proving into a Flutter app, allowing for streamlined creation of ZK-enabled mobile apps.

> [!IMPORTANT]
> Please refer to the latest [mopro documentation](https://zkmopro.org/docs/next/setup/flutter-setup) for the most up-to-date information.

## Running The Example App

### Prerequisites

1. **Install Flutter**

   If Flutter is not already installed, you can follow the [official Flutter installation guide](https://docs.flutter.dev/get-started/install) for your operating system.

2. **Check Flutter Environment**

   After installing Flutter, verify that your development environment is properly set up by running the following command in your terminal:

   ```bash
   flutter doctor
   ```

   This command will identify any missing dependencies or required configurations.

3. **Install Flutter Dependencies**

   Navigate to the root directory of the project in your terminal and run:

   ```bash
   flutter pub get
   ```

   This will install the necessary dependencies for the project.

### Running the App via VS Code

1. Open the project in VS Code.
2. Open the "Run and Debug" panel.
3. Start an emulator (iOS/Android) or connect your physical device.
4. Select "example" in the run menu and press "Run".

### Running the App via Console

If you prefer using the terminal to run the app, use the following steps:

1. For Android:

   Ensure you have an Android emulator running or a device connected. Then run:

   ```bash
   flutter run
   ```

2. For iOS:

   Make sure you have an iOS simulator running or a device connected. Then run:

   ```bash
   flutter run
   ```

## Integrating Your ZKP

The example app comes with a simple prover generated from a Circom circuit. To integrate your own prover, follow the steps below.

### Setup

Follow the [Rust Setup steps from the MoPro official docs](https://zkmopro.org/docs/setup/rust-setup) to generate the platform-specific libraries.

### Copying The Generated Libraries

#### iOS

1. Replace `mopro.swift` at [`mopro_flutter_plugin/ios/Classes/mopro.swift`](mopro_flutter_plugin/ios/Classes/mopro.swift) with the file generated during the [Setup](#setup).
2. Replace the directory [`mopro_flutter_plugin/ios/MoproBindings.xcframework`](mopro_flutter_plugin/ios/MoproBindings.xcframework) with the one generated during the [Setup](#setup).

#### Android

1. Replace the directory [`mopro_flutter_plugin/android/src/main/jniLibs`](mopro_flutter_plugin/android/src/main/jniLibs) with the one generated during the [Setup](#setup).
2. Replace `mopro.kt` at [`mopro_flutter_plugin/android/src/main/kotlin/uniffi/mopro/mopro.kt`](mopro_flutter_plugin/android/src/main/kotlin/uniffi/mopro/mopro.kt) with the file generated during the [Setup](#setup).

### zKey

1. Place your `.zkey` file in your app's assets folder. For example, to run the included example app, you need to replace the `.zkey` at [`assets/multiplier2_final.zkey`](assets/multiplier2_final.zkey) with your file. If you change the `.zkey` file name, don't forget to update the asset definition in your app's [`pubspec.yaml`](pubspec.yaml):

   ```yaml
   assets:
     - assets/your_new_zkey_file.zkey
   ```

2. Load the new `.zkey` file properly in your Dart code. For example, update the file path in [`lib/main.dart`](lib/main.dart):

   ```dart
   var inputs = "{\"a\":[\"3\"],\"b\":[\"5\"]}";
   proofResult = await _moproFlutterPlugin.generateProof("assets/multiplier2_final.zkey", inputs);
   ```

Don't forget to modify the input values for your specific case!

## Developing The Plugin

### Android

Open the `./android` directory in Android Studio. You will be able to browse to the plugin code in `Android` and `Project` view.
