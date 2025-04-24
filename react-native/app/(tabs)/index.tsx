import { StyleSheet, Button, View, Text } from "react-native";

import { proveZkemail, verifyZkemail } from "@/modules/mopro";
import * as FileSystem from "expo-file-system";
import { useState } from "react";
import { input } from "./Input";

export default function HomeScreen() {
    const [proof, setProof] = useState<Uint8Array>(new Uint8Array());
    const [disabled, setDisabled] = useState(false);
    const [verifyDisabled, setVerifyDisabled] = useState(true);
    const [verified, setVerified] = useState(false);
    const [time, setTime] = useState(0);
    async function downloadSrs(): Promise<string> {
        const newSrsName = "srs.local";
        const srsUrl =
            "https://github.com/zkmopro/mopro-zkemail-nr/raw/refs/heads/main/public/srs.local";
        const newSrsPath = `${FileSystem.documentDirectory}${newSrsName}`;
        const srsInfo = await FileSystem.getInfoAsync(newSrsPath);
        if (!srsInfo.exists) {
            try {
                const downloadResult = await FileSystem.downloadAsync(
                    srsUrl,
                    newSrsPath
                );
                console.log("File downloaded to:", downloadResult.uri);
            } catch (error) {
                console.error("Failed to download file:", error);
            }
        }
        return newSrsPath;
    }
    async function genProof(): Promise<void> {
        setDisabled(true);
        const srsPath = await downloadSrs();
        const circuitInputs = input;
        try {
            const startTime = new Date();
            const res = proveZkemail(
                srsPath.replace("file://", ""),
                circuitInputs
            );
            const endTime = new Date();
            setProof(res);
            setTime(endTime.getTime() - startTime.getTime());
            setVerifyDisabled(false);
        } catch (error) {
            console.error("Failed to prove zkemail:", error);
        }
        setDisabled(false);
    }
    async function verifyProof(): Promise<void> {
        const srsPath = await downloadSrs();
        const res = verifyZkemail(srsPath.replace("file://", ""), proof);
        setVerified(res);
    }
    return (
        <View style={{ padding: 100 }}>
            <Text style={styles.title}>zkemail proof</Text>
            <Button
                title="Proof"
                disabled={disabled}
                onPress={() => genProof()}
            />
            <Button
                title="Verify"
                disabled={verifyDisabled}
                onPress={() => verifyProof()}
            />
            <Text style={styles.stepContainer}>
                <Text style={styles.label}>Time:</Text> {time} ms
            </Text>
            <View style={styles.stepContainer}>
                <Text style={styles.label}>
                    Verified: {verified ? "Verified" : ""}
                </Text>
            </View>
        </View>
    );
}

const styles = StyleSheet.create({
    stepContainer: {
        gap: 8,
        marginBottom: 8,
    },
    title: {
        fontSize: 24,
        fontWeight: "bold",
        marginBottom: 16,
    },
    input: {
        height: 40,
        borderColor: "gray",
        borderWidth: 1,
        flex: 1,
        paddingHorizontal: 10,
    },
    inputContainer: {
        flexDirection: "row",
        alignItems: "center",
        marginBottom: 10,
    },
    label: {
        fontSize: 16,
        marginRight: 10,
    },
    reactLogo: {
        height: 178,
        width: 290,
        bottom: 0,
        left: 0,
        position: "absolute",
    },
    output: {
        fontSize: 16,
        borderColor: "gray",
        borderWidth: 1,
        padding: 10,
    },
});
