import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import '../styles/globals.css';
import { Providers } from './providers';

/**
 * DOCTRINE ALIGNMENT: Π (Product Integration)
 * Root layout component with OpenTelemetry instrumentation
 */

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
  title: 'YAWL Process Editor',
  description: 'Next.js-based YAWL workflow process editor with RDF/Turtle support',
  keywords: ['YAWL', 'workflow', 'process editor', 'RDF', 'Turtle', 'ontology'],
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={inter.className}>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
