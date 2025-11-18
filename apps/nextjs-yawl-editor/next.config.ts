import type { NextConfig } from 'next';

/**
 * DOCTRINE ALIGNMENT: Π (Product Integration)
 * Next.js configuration for YAWL Process Editor
 * Ensures proper module resolution and RDF/Turtle support
 */

const nextConfig: NextConfig = {
  /* Enable React strict mode for better development experience */
  reactStrictMode: true,

  /* Experimental features */
  experimental: {
    /* Enable Server Actions for API routes */
    serverActions: {
      bodySizeLimit: '2mb',
    },
    /* Optimize package imports */
    optimizePackageImports: [
      'lucide-react',
      '@radix-ui/react-icons',
      'reactflow',
    ],
  },

  /* Webpack configuration for RDF/Turtle modules */
  webpack: (config, { isServer }) => {
    /* Handle .ttl files as raw text */
    config.module.rules.push({
      test: /\.ttl$/,
      type: 'asset/source',
    });

    /* Handle N3 library properly */
    if (!isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        net: false,
        tls: false,
      };
    }

    return config;
  },

  /* Headers for CORS and security */
  async headers() {
    return [
      {
        source: '/api/:path*',
        headers: [
          { key: 'Access-Control-Allow-Credentials', value: 'true' },
          { key: 'Access-Control-Allow-Origin', value: '*' },
          { key: 'Access-Control-Allow-Methods', value: 'GET,POST,PUT,DELETE,OPTIONS' },
          { key: 'Access-Control-Allow-Headers', value: 'Content-Type, Authorization' },
        ],
      },
    ];
  },

  /* Environment variables to expose to client */
  env: {
    NEXT_PUBLIC_APP_NAME: 'YAWL Process Editor',
    NEXT_PUBLIC_APP_VERSION: '0.1.0',
  },
};

export default nextConfig;
