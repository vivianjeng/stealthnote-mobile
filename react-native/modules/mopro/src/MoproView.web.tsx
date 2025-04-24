import * as React from 'react';

import { MoproViewProps } from './Mopro.types';

export default function MoproView(props: MoproViewProps) {
  return (
    <div>
      <span>{props.name}</span>
    </div>
  );
}
