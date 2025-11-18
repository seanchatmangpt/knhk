/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,

  // Enable server actions
  experimental: {
    serverActions: {
      allowedOrigins: ['localhost:3000'],
    },
  },

  // Webpack configuration for RDF libraries
  webpack: (config) => {
    // Handle .ttl files as text
    config.module.rules.push({
      test: /\.ttl$/,
      type: 'asset/source',
    });

    return config;
  },

  // Environment variables
  env: {
    NEXT_PUBLIC_OTEL_EXPORTER_URL:
      process.env.NEXT_PUBLIC_OTEL_EXPORTER_URL || 'http://localhost:4318/v1/traces',
  },
};

module.exports = nextConfig;
