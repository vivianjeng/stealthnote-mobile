import { NativeModulesProxy, EventEmitter, Subscription } from 'expo-modules-core';

// Import the native module. On web, it will be resolved to Mopro.web.ts
// and on native platforms to Mopro.ts
import MoproModule from './src/MoproModule';
import MoproView from './src/MoproView';
import { ChangeEventPayload, MoproViewProps } from './src/Mopro.types';

// Get the native constant value.
export const PI = MoproModule.PI;

export function hello(): string {
  return MoproModule.hello();
}

export function proveZkemail(srsPath: string, inputs: { [key: string]: string[] }): Uint8Array {
  return MoproModule.proveZkemail(srsPath, inputs);
}

export function verifyZkemail(srsPath: string, proof: Uint8Array): boolean {
  return MoproModule.verifyZkemail(srsPath, proof);
}

export async function setValueAsync(value: string) {
  return await MoproModule.setValueAsync(value);
}

const emitter = new EventEmitter(MoproModule ?? NativeModulesProxy.Mopro);

export function addChangeListener(listener: (event: ChangeEventPayload) => void): Subscription {
  return emitter.addListener<ChangeEventPayload>('onChange', listener);
}

export { MoproView, MoproViewProps, ChangeEventPayload };
