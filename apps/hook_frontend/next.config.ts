import type { NextConfig } from 'next';

// ----------------------------------------------------------------------

/**
 * Static Exports in Next.js
 *
 * 1. Set `isStaticExport = true` in `next.config.{mjs|ts}`.
 * 2. This allows `generateStaticParams()` to pre-render dynamic routes at build time.
 *
 * For more details, see:
 * https://nextjs.org/docs/app/building-your-application/deploying/static-exports
 *
 * NOTE: Remove all "generateStaticParams()" functions if not using static exports.
 */
const isStaticExport = process.env.BUILD_STATIC_EXPORT === 'true';
const backendUrl = process.env.HOOK_BACKEND_URL ?? 'http://127.0.0.1:5555';

// ----------------------------------------------------------------------

const backendRewrites = isStaticExport
  ? {}
  : {
      async rewrites() {
        return [
          {
            source: '/v1/:path*',
            destination: `${backendUrl}/v1/:path*`,
          },
          {
            source: '/v1beta/:path*',
            destination: `${backendUrl}/v1beta/:path*`,
          },
        ];
      },
    };

const nextConfig: NextConfig = {
  trailingSlash: true,
  output: isStaticExport ? 'export' : undefined,
  env: {
    BUILD_STATIC_EXPORT: JSON.stringify(isStaticExport),
  },
  ...backendRewrites,
  // Without --turbopack (next dev)
  webpack(config) {
    config.module.rules.push({
      test: /\.svg$/,
      use: ['@svgr/webpack'],
    });

    return config;
  },
  // With --turbopack (next dev --turbopack)
  turbopack: {
    rules: {
      '*.svg': {
        loaders: ['@svgr/webpack'],
        as: '*.js',
      },
    },
  },
};

export default nextConfig;
