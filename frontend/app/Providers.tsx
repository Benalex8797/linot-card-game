"use client";

export default function Providers({ children }: { children: React.ReactNode }) {
  // No providers needed - config loads client-side in components
  return <>{children}</>;
}
