import type {NextConfig} from "next";

const nextConfig: NextConfig = {
  output: "export", // Static export
  images: {
    unoptimized: true,
  },
  // Read env vars at build time
  env: {
    NEXT_PUBLIC_APP_ID: process.env.NEXT_PUBLIC_APP_ID,
    NEXT_PUBLIC_PLAY_CHAIN: process.env.NEXT_PUBLIC_PLAY_CHAIN,
    NEXT_PUBLIC_USER_CHAIN_1: process.env.NEXT_PUBLIC_USER_CHAIN_1,
    NEXT_PUBLIC_USER_CHAIN_2: process.env.NEXT_PUBLIC_USER_CHAIN_2,
    NEXT_PUBLIC_GRAPHQL_PORT_1: process.env.NEXT_PUBLIC_GRAPHQL_PORT_1,
    NEXT_PUBLIC_GRAPHQL_PORT_2: process.env.NEXT_PUBLIC_GRAPHQL_PORT_2,
  },
};

export default nextConfig;
