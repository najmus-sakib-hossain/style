import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  images: {
    domains: ['assets.codepen.io', "fastly.picsum.photos"],
  },
  turbopack: {
    root: process.cwd(),
  },
};

export default nextConfig;
