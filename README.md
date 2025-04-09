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
- noir = { path = "../noir-rs/noir", features = ["barretenberg"] }
+ noir = { path = "../noir-rs/noir", features = ["barretenberg", "android-compat"] }
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
cp -r MoproiOSBindings ios/MoproiOSBindings
```

### Android

Copy the generated files into your Android project:

```sh
cp -r MoproAndroidBindings/uniffi android/app/src/main/java/uniffi
cp -r MoproAndroidBindings/jniLibs android/app/src/main/jniLibs
```

## 📂 Open the project

Follow the instructions to open the development tools

For iOS:

```sh
open ios/MoproApp.xcodeproj
```

For Android:

```sh
open android -a Android\ Studio
```
