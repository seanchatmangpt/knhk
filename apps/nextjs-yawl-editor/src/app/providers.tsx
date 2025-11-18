'use client';

import { ReactNode } from 'react';

/**
 * DOCTRINE ALIGNMENT: Σ (Sum of All Fears - System Integrity)
 * Provider wrapper for client-side state management and context
 */

interface ProvidersProps {
  children: ReactNode;
}

export function Providers({ children }: ProvidersProps) {
  return <>{children}</>;
}
