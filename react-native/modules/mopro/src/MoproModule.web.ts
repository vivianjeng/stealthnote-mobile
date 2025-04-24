import { EventEmitter } from 'expo-modules-core';
import { G1, G2, ProofCalldata, Result } from '..';
import { Buffer } from 'buffer';

const emitter = new EventEmitter({} as any);

if (typeof process === 'undefined') {
  global.process = { browser: true }; // Define process.browser for the web
} else if (typeof process.browser === 'undefined') {
  process.browser = true; // Define process.browser if it's not already defined
}

export default {
  PI: Math.PI,
  async setValueAsync(value: string): Promise<void> {
    emitter.emit('onChange', { value });
  },
  async generateCircomProofWeb(wasmPath: string, zkeyPath: string, circuitInputs: any): Promise<Result> {

    Buffer.from('anything', 'base64');
    window.Buffer = window.Buffer || require("buffer").Buffer;
    const snarkjs = require('snarkjs');

    const wasm = await fetch(wasmPath).then((r) => r.arrayBuffer());
    const zkey = await fetch(zkeyPath).then((r) => r.arrayBuffer());

    const snarkjsProof = await snarkjs.groth16.fullProve(circuitInputs, new Uint8Array(wasm), new Uint8Array(zkey))
    const a: G1 = {
      x: snarkjsProof.proof.pi_a[0],
      y: snarkjsProof.proof.pi_a[1],
    }
    const b: G2 = {
      x: snarkjsProof.proof.pi_b[0],
      y: snarkjsProof.proof.pi_b[1],
    }
    const c: G1 = {
      x: snarkjsProof.proof.pi_c[0],
      y: snarkjsProof.proof.pi_c[1],
    }
    const proof: ProofCalldata = {
      a: a,
      b: b,
      c: c,
    }
    const inputs: string[] = snarkjsProof.publicSignals;
    const result: Result = {
      proof: proof,
      inputs: inputs,
    }
    return result;
  },
  hello() {
    return 'Hello world! 👋';
  },
};
