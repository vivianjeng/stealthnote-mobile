//
//  ContentView.swift
//  MoproApp
//
import SwiftUI
import moproFFI

// Structs for decoding zkemail_input.json
struct ZkEmailInputTest: Decodable {
    let header: HeaderTest
    let pubkey: PubKeyTest
    let signature: [String]
    let date_index: UInt32
    let subject_sequence: SequenceTest
    let from_header_sequence: SequenceTest
    let from_address_sequence: SequenceTest
}

struct HeaderTest: Decodable {
    let storage: [UInt8]
    let len: UInt32
}

struct PubKeyTest: Decodable {
    let modulus: [String]
    let redc: [String]
}

struct SequenceTest: Decodable {
    let index: UInt32
    let length: UInt32
}

struct ContentView: View {
    @State private var textViewText = ""
    @State private var isNoirProveButtonEnabled = true
    @State private var zkEmailProof: Data?
    
    var body: some View {
        VStack(spacing: 10) {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Button("Prove Noir", action: runNoirProveAction).disabled(!isNoirProveButtonEnabled).accessibilityIdentifier("proveNoir")
            Button("zkEmail Prove", action: runZkEmailProveAction).accessibilityIdentifier("proveZkEmail")
            Button("zkEmail Verify", action: runZkEmailVerifyAction).accessibilityIdentifier("verifyZkEmail").disabled(zkEmailProof == nil)

            ScrollView {
                Text(textViewText)
                    .padding()
                    .accessibilityIdentifier("proof_log")
            }
            .frame(height: 200)
        }
        .padding()
    }
}

extension ContentView {
    func runNoirProveAction() {
        isNoirProveButtonEnabled = false
        textViewText += "Generating Noir proof... "
        do {
            // Prepare inputs
            let a = 3
            let b = 5
            let c = a*b
            let input_str: String = "{\"b\":[\"5\"],\"a\":[\"3\"]}"

            // Expected outputs
            let outputs: [String] = [String(c), String(a)]
            
            let start = CFAbsoluteTimeGetCurrent()
            let valid = prove()
            print(valid)
            
            let end = CFAbsoluteTimeGetCurrent()
            let timeTaken = end - start
            
            
            textViewText += "\(String(format: "%.3f", timeTaken))s 1️⃣\n"
            
            isNoirProveButtonEnabled = true
        } catch {
            textViewText += "\nProof generation failed: \(error.localizedDescription)\n"
        }
    }

    func runZkEmailProveAction() {
        textViewText += "Generating zkEmail proof...\n"
        
        // Get the path to the SRS file in the app bundle
        guard let srsPath = Bundle.main.path(forResource: "srs", ofType: "local") else {
            textViewText += "Error: Could not find SRS file in app bundle\n"
            return
        }
        
        // Get the path to the input JSON file
        guard let inputJsonPath = Bundle.main.path(forResource: "zkemail_input", ofType: "json") else {
            textViewText += "Error: Could not find zkemail_input.json in app bundle\n"
            return
        }
        
        // Load and parse JSON
        do {
            let jsonData = try Data(contentsOf: URL(fileURLWithPath: inputJsonPath))
            let decoder = JSONDecoder()
            let inputData = try decoder.decode(ZkEmailInputTest.self, from: jsonData)
            
            // Convert to the format expected by proveZkemail
            var inputs: [String: [String]] = [:]
            
            // Header storage - convert [UInt8] to [String]
            inputs["header_storage"] = inputData.header.storage.map { String($0) }
            inputs["header_len"] = [String(inputData.header.len)]
            
            // Public key
            inputs["pubkey_modulus"] = inputData.pubkey.modulus
            inputs["pubkey_redc"] = inputData.pubkey.redc
            
            // Signature
            inputs["signature"] = inputData.signature
            
            // Indexes and lengths
            inputs["date_index"] = [String(inputData.date_index)]
            inputs["subject_index"] = [String(inputData.subject_sequence.index)]
            inputs["subject_length"] = [String(inputData.subject_sequence.length)]
            inputs["from_header_index"] = [String(inputData.from_header_sequence.index)]
            inputs["from_header_length"] = [String(inputData.from_header_sequence.length)]
            inputs["from_address_index"] = [String(inputData.from_address_sequence.index)]
            inputs["from_address_length"] = [String(inputData.from_address_sequence.length)]
            
            // Clear previous proof
            zkEmailProof = nil
            
            // Run in background thread
            DispatchQueue.global(qos: .userInitiated).async {
                let start = CFAbsoluteTimeGetCurrent()
                do {
                    // Generate the proof
                    let proofData = try! proveZkemail(srsPath: srsPath, inputs: inputs)
                    
                    let end = CFAbsoluteTimeGetCurrent()
                    let timeTaken = end - start
                    
                    // Update UI on the main thread
                    DispatchQueue.main.async {
                        self.zkEmailProof = proofData
                        self.textViewText += "Proof generated successfully! (took \(String(format: "%.3f", timeTaken))s) ✅\n"
                    }
                } catch {
                    DispatchQueue.main.async {
                        self.textViewText += "Proof generation failed: \(error.localizedDescription)\n"
                    }
                }
            }
            
        } catch {
            textViewText += "Error loading or parsing input JSON: \(error.localizedDescription)\n"
        }
    }

    func runZkEmailVerifyAction() {
        guard let proofData = zkEmailProof else {
            textViewText += "Error: Proof data is not available. Generate proof first.\n"
            return
        }

        textViewText += "Verifying zkEmail proof...\n"
        
        // Get the path to the SRS file in the app bundle
        guard let srsPath = Bundle.main.path(forResource: "srs", ofType: "local") else {
            textViewText += "Error: Could not find SRS file in app bundle\n"
            return
        }

        DispatchQueue.global(qos: .userInitiated).async {
            let start = CFAbsoluteTimeGetCurrent()
            do {
                // Verify the proof
                let isValid = try! verifyZkemail(srsPath: srsPath, proof: proofData)

                let end = CFAbsoluteTimeGetCurrent()
                let timeTaken = end - start

                DispatchQueue.main.async {
                    self.textViewText += "Verification result: \(isValid) (took \(String(format: "%.3f", timeTaken))s) ✅\n"
                }
            } catch {
                DispatchQueue.main.async {
                    self.textViewText += "Verification failed: \(error.localizedDescription)\n"
                }
            }
        }
    }
}

