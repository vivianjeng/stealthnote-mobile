# Noir with Mopro

This project demonstrates how to integrate a Noir circuit with the mopro CLI to generate platform bindings.
See how to install mopro CLI: [Getting started](https://zkmopro.org/docs/getting-started).

## 🔧 Build the Bindings

### iOS

Run

```sh
mopro build
```

and select `aarch64-apple-ios`

### Android

Activate `android-compat` feature in [Cargo.toml](./Cargo.toml).

```diff
- noir = { git = "https://github.com/zkmopro/noir-rs", features = ["barretenberg"] }
+ noir = { git = "https://github.com/zkmopro/noir-rs", features = ["barretenberg", "android-compat"] }
```

Run

```sh
mopro build
```

and select `aarch64-linux-android`

## 🔄 Manually Update Bindings

### iOS

Copy the generated `MoproiOSBindings` directory into your iOS project:

```sh
cp -r MoproiOSBindings ios
```

### Android

Copy the generated files into your Android project:

```sh
cp -r MoproAndroidBindings/uniffi android/app/src/main/java
cp -r MoproAndroidBindings/jniLibs android/app/src/main
```

### React Native

Copy the generated files into your React Native project:

```sh
cp -r MoproiOSBindings react-native/modules/mopro/ios && \
cp -r MoproAndroidBindings/uniffi react-native/modules/mopro/android/src/main/java && \
cp -r MoproAndroidBindings/jniLibs react-native/modules/mopro/android/src/main 
```

## 📂 Open the project

Follow the instructions to open the development tools

### iOS

```sh
open ios/MoproApp.xcodeproj
```

### Android

```sh
open android -a Android\ Studio
```

### React Native

```sh
cd react-native && npm install
```

For iOS device:

```sh
npm run ios
```

For Android device/simulator:

```sh
npm run android
```

### Flutter

- Check flutter environment
   ```sh
   flutter doctor 
   ```

- Install Flutter Dependencies
   ```sh
   flutter pub get
   ```

- Run the app (Please turn on emulators before running the command)
   ```sh
   flutter run
   ```

## 📊 Benchmarks

The following benchmarks were conducted on Apple M3 chips in release mode:

| zkEmail Operation | iOS, Time (ms) | Android, Time (ms) |
|-----------|---------------|-------------------|
| Proof Generation | 1309 | 3826 |
| Verification | 962 | 2857 |
