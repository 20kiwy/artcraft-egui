import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "standalone",
  /* config options here */
  typescript: {
    ignoreBuildErrors: true,
  },
  reactStrictMode: false,
  // Note: src/instrumentation.ts spawns artcraft-proxy as a child of this
  // server so the preview URL can act as the egui app's proxy URL (the
  // gateway only exposes port 3000).
};

export default nextConfig;
