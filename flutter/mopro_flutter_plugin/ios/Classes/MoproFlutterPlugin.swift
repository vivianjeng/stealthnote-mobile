import Flutter
import moproFFI
import UIKit

public class MoproFlutterPlugin: NSObject, FlutterPlugin {
    public static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(name: "mopro_flutter", binaryMessenger: registrar.messenger())
    let instance = MoproFlutterPlugin()
    registrar.addMethodCallDelegate(instance, channel: channel)
  }

  public func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    // Run potentially long-running tasks on a background thread
    DispatchQueue.global(qos: .userInitiated).async {
        let dispatchResult: (Any?) -> Void = { response in
            DispatchQueue.main.async {
                result(response)
            }
        }
        let dispatchError: (String, String?, Any?) -> Void = { code, message, details in
             DispatchQueue.main.async {
                result(FlutterError(code: code, message: message, details: details))
            }
        }

        switch call.method {
        case "getPlatformVersion":
          dispatchResult("iOS " + UIDevice.current.systemVersion)
        case "getApplicationDocumentsDirectory":
             do {
                 let documentsPath = try FileManager.default.url(for: .documentDirectory,
                                                                in: .userDomainMask,
                                                                appropriateFor: nil,
                                                                create: false).path
                 dispatchResult(documentsPath)
             } catch {
                 dispatchError("DIR_ERROR", "Could not get documents directory", error.localizedDescription)
             }
        case "proveZkEmail":
            guard let args = call.arguments as? [String: Any],
                  let srsPath = args["srsPath"] as? String,
                  let inputs = args["inputs"] as? [String: [String]] else {
                dispatchError("INVALID_ARGUMENTS", "srsPath or inputs is null or invalid format", nil)
                return
            }
            do {
                // Note: The mopro.swift binding `proveZkemail` doesn't throw currently,
                // but we include a do-catch for future-proofing and consistency.
                let proofBytes = proveZkemail(srsPath: srsPath, inputs: inputs)
                // Wrap the result in the format expected by Dart
                let resultMap: [String: Any?] = ["proof": FlutterStandardTypedData(bytes: proofBytes), "error": nil]
                dispatchResult(resultMap)
            } catch {
                // If proveZkemail were to throw, handle it here
                 let resultMap: [String: Any?] = ["proof": nil, "error": error.localizedDescription]
                 dispatchResult(resultMap)
            }
        case "verifyZkEmail":
             guard let args = call.arguments as? [String: Any],
                  let srsPath = args["srsPath"] as? String,
                  let proofData = args["proof"] as? FlutterStandardTypedData else {
                dispatchError("INVALID_ARGUMENTS", "srsPath or proof is null or invalid format", nil)
                return
            }
            do {
                 // Note: The mopro.swift binding `verifyZkemail` doesn't throw currently.
                let isValid = verifyZkemail(srsPath: srsPath, proof: proofData.data)
                 // Wrap the result in the format expected by Dart
                let resultMap: [String: Any?] = ["isValid": isValid, "error": nil]
                dispatchResult(resultMap)
            } catch {
                // If verifyZkemail were to throw, handle it here
                let resultMap: [String: Any?] = ["isValid": false, "error": error.localizedDescription]
                 dispatchResult(resultMap)
            }
        default:
            dispatchResult(FlutterMethodNotImplemented)
        }
    } // End of DispatchQueue.global().async
  }
}
