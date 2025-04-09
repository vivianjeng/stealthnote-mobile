//
//  ContentView.swift
//  MoproApp
//
import SwiftUI
import moproFFI


struct ContentView: View {
    @State private var textViewText = ""
    @State private var isNoirProveButtonEnabled = true
    
    var body: some View {
        VStack(spacing: 10) {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Button("Prove Noir", action: runNoirProveAction).disabled(!isNoirProveButtonEnabled).accessibilityIdentifier("proveNoir")

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
}

