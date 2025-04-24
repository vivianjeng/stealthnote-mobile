import { requireNativeViewManager } from 'expo-modules-core';
import * as React from 'react';

import { MoproViewProps } from './Mopro.types';

const NativeView: React.ComponentType<MoproViewProps> =
  requireNativeViewManager('Mopro');

export default function MoproView(props: MoproViewProps) {
  return <NativeView {...props} />;
}
