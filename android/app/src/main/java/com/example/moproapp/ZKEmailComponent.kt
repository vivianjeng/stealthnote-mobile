package com.example.moproapp

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.json.JSONObject
import uniffi.mopro.proveZkemail
import uniffi.mopro.verifyZkemail
import java.io.File
import java.io.InputStream

@Composable
fun ZKEmailComponent() {
    val context = LocalContext.current
    var provingTime by remember { mutableStateOf("") }
    var proofResult by remember { mutableStateOf("") }
    var verificationTime by remember { mutableStateOf("") }
    var verificationResult by remember { mutableStateOf("") }
    var proofBytes by remember { mutableStateOf<ByteArray?>(null) }
    
    // Status states
    var isGeneratingProof by remember { mutableStateOf(false) }
    var isVerifyingProof by remember { mutableStateOf(false) }
    var statusMessage by remember { mutableStateOf("Ready to generate proof") }

    // Function to prepare zkemail inputs from JSON file
    fun prepareZkEmailInputs(): Map<String, List<String>> {
        try {
            // Copy zkemail_input.json from assets to app directory if needed
            val jsonFile = File(context.filesDir, "zkemail_input.json")
            if (!jsonFile.exists()) {
                context.assets.open("zkemail_input.json").use { input ->
                    jsonFile.outputStream().use { output ->
                        input.copyTo(output)
                    }
                }
            }

            // Read the JSON content
            val jsonContent = context.assets.open("zkemail_input.json").bufferedReader().use { it.readText() }
            val jsonObject = JSONObject(jsonContent)

            // Parse the input data according to the expected format
            val inputs = HashMap<String, List<String>>()
            
            // Add header storage
            val headerStorage = jsonObject.getJSONObject("header").getJSONArray("storage")
            val headerStorageList = mutableListOf<String>()
            for (i in 0 until headerStorage.length()) {
                headerStorageList.add(headerStorage.getInt(i).toString())
            }
            inputs["header_storage"] = headerStorageList
            
            // Add header length
            inputs["header_len"] = listOf(jsonObject.getJSONObject("header").getInt("len").toString())
            
            // Add pubkey modulus
            val pubkeyModulus = jsonObject.getJSONObject("pubkey").getJSONArray("modulus")
            val pubkeyModulusList = mutableListOf<String>()
            for (i in 0 until pubkeyModulus.length()) {
                pubkeyModulusList.add(pubkeyModulus.getString(i))
            }
            inputs["pubkey_modulus"] = pubkeyModulusList
            
            // Add pubkey redc
            val pubkeyRedc = jsonObject.getJSONObject("pubkey").getJSONArray("redc")
            val pubkeyRedcList = mutableListOf<String>()
            for (i in 0 until pubkeyRedc.length()) {
                pubkeyRedcList.add(pubkeyRedc.getString(i))
            }
            inputs["pubkey_redc"] = pubkeyRedcList
            
            // Add signature
            val signature = jsonObject.getJSONArray("signature")
            val signatureList = mutableListOf<String>()
            for (i in 0 until signature.length()) {
                signatureList.add(signature.getString(i))
            }
            inputs["signature"] = signatureList
            
            // Add date index
            inputs["date_index"] = listOf(jsonObject.getInt("date_index").toString())
            
            // Add subject sequence
            inputs["subject_index"] = listOf(jsonObject.getJSONObject("subject_sequence").getInt("index").toString())
            inputs["subject_length"] = listOf(jsonObject.getJSONObject("subject_sequence").getInt("length").toString())
            
            // Add from header sequence
            inputs["from_header_index"] = listOf(jsonObject.getJSONObject("from_header_sequence").getInt("index").toString())
            inputs["from_header_length"] = listOf(jsonObject.getJSONObject("from_header_sequence").getInt("length").toString())
            
            // Add from address sequence
            inputs["from_address_index"] = listOf(jsonObject.getJSONObject("from_address_sequence").getInt("index").toString())
            inputs["from_address_length"] = listOf(jsonObject.getJSONObject("from_address_sequence").getInt("length").toString())
            
            return inputs
        } catch (e: Exception) {
            e.printStackTrace()
            // Return empty map on error
            return emptyMap()
        }
    }
    
    // Function to ensure SRS file is available
    fun prepareSrsFile(): String {
        val srsFile = File(context.filesDir, "srs.local")
        if (!srsFile.exists()) {
            try {
                context.assets.open("srs.local").use { input ->
                    srsFile.outputStream().use { output ->
                        input.copyTo(output)
                    }
                }
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
        return srsFile.absolutePath
    }

    Box(modifier = Modifier.fillMaxSize().padding(16.dp), contentAlignment = Alignment.Center) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(
                text = "zkEmail (Noir)",
                modifier = Modifier.padding(bottom = 20.dp),
                fontWeight = FontWeight.Bold,
                fontSize = 22.sp
            )
            
            // Status message with prominent styling
            Text(
                text = statusMessage,
                modifier = Modifier.padding(bottom = 24.dp),
                textAlign = TextAlign.Center,
                fontSize = 16.sp,
                fontWeight = if (isGeneratingProof || isVerifyingProof) FontWeight.Bold else FontWeight.Normal
            )
            
            // Progress indicator when operations are running
            if (isGeneratingProof || isVerifyingProof) {
                CircularProgressIndicator(
                    modifier = Modifier.padding(bottom = 16.dp)
                )
            }
            
            Button(
                onClick = {
                    isGeneratingProof = true
                    provingTime = ""
                    proofResult = ""
                    statusMessage = "Generating proof... This may take some time"
                    
                    Thread(
                        Runnable {
                            try {
                                val srsPath = prepareSrsFile()
                                val inputs = prepareZkEmailInputs()
                                
                                val startTime = System.currentTimeMillis()
                                proofBytes = proveZkemail(srsPath, inputs)
                                val endTime = System.currentTimeMillis()
                                val duration = endTime - startTime
                                
                                provingTime = "Proving time: $duration ms"
                                proofResult = "Proof generated: ${proofBytes?.size ?: 0} bytes"
                                statusMessage = "Proof generation completed"
                            } catch (e: Exception) {
                                provingTime = "Proving failed"
                                proofResult = "Error: ${e.message}"
                                statusMessage = "Proof generation failed"
                                e.printStackTrace()
                            } finally {
                                isGeneratingProof = false
                            }
                        }
                    ).start()
                },
                modifier = Modifier.padding(top = 8.dp).width(220.dp),
                enabled = !isGeneratingProof && !isVerifyingProof
            ) { 
                Text(text = "Generate zkEmail Proof")
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            
            Button(
                onClick = {
                    isVerifyingProof = true
                    verificationTime = ""
                    verificationResult = ""
                    statusMessage = "Verifying proof..."
                    
                    Thread(
                        Runnable {
                            try {
                                proofBytes?.let { proof ->
                                    val srsPath = prepareSrsFile()
                                    
                                    val startTime = System.currentTimeMillis()
                                    val result = verifyZkemail(srsPath, proof)
                                    val endTime = System.currentTimeMillis()
                                    val duration = endTime - startTime
                                    
                                    verificationTime = "Verification time: $duration ms"
                                    verificationResult = "Verification result: $result"
                                    statusMessage = if (result) "Proof verified successfully!" else "Proof verification failed!"
                                } ?: run {
                                    verificationResult = "No proof available"
                                    statusMessage = "Please generate a proof first"
                                }
                            } catch (e: Exception) {
                                verificationTime = "Verification failed"
                                verificationResult = "Error: ${e.message}"
                                statusMessage = "Proof verification error"
                                e.printStackTrace()
                            } finally {
                                isVerifyingProof = false
                            }
                        }
                    ).start()
                },
                modifier = Modifier.padding(top = 8.dp).width(220.dp),
                enabled = !isGeneratingProof && !isVerifyingProof && proofBytes != null
            ) { 
                Text(text = "Verify zkEmail Proof") 
            }

            Spacer(modifier = Modifier.height(40.dp))
            
            // Results displayed in a more organized way
            if (provingTime.isNotEmpty() || proofResult.isNotEmpty() || 
                verificationTime.isNotEmpty() || verificationResult.isNotEmpty()) {
                
                Text(
                    text = "Results",
                    fontWeight = FontWeight.Bold,
                    fontSize = 18.sp,
                    modifier = Modifier.padding(bottom = 8.dp)
                )
                
                if (provingTime.isNotEmpty()) {
                    Text(
                        text = provingTime,
                        modifier = Modifier.padding(top = 4.dp).width(280.dp),
                        textAlign = TextAlign.Center
                    )
                }
                
                if (proofResult.isNotEmpty()) {
                    Text(
                        text = proofResult,
                        modifier = Modifier.padding(top = 4.dp).width(280.dp),
                        textAlign = TextAlign.Center
                    )
                }
                
                if (verificationTime.isNotEmpty()) {
                    Text(
                        text = verificationTime,
                        modifier = Modifier.padding(top = 4.dp).width(280.dp),
                        textAlign = TextAlign.Center
                    )
                }
                
                if (verificationResult.isNotEmpty()) {
                    Text(
                        text = verificationResult,
                        modifier = Modifier.padding(top = 4.dp).width(280.dp),
                        textAlign = TextAlign.Center,
                        fontWeight = if (verificationResult.contains("true")) FontWeight.Bold else FontWeight.Normal
                    )
                }
            }
        }
    }
} 