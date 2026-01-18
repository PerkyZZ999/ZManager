/**
 * SvgIcon component for loading external SVG icons
 *
 * Uses <img> for external SVG files with proper sizing and fallback.
 * Applies invert filter by default for dark theme compatibility.
 */

import { memo, useState } from "react";

export interface SvgIconProps {
  /** Path to the SVG file (relative to public/) */
  src: string;
  /** Size in pixels (width and height) */
  size?: number;
  /** Additional CSS classes */
  className?: string;
  /** Alt text for accessibility */
  alt?: string;
  /** Fallback icon path if the main one fails to load */
  fallback?: string;
  /** Whether to invert colors for dark theme (default: true for ui icons, false for others) */
  invert?: boolean;
}

/**
 * Renders an external SVG icon with proper sizing and error handling
 */
export const SvgIcon = memo(function SvgIcon({
  src,
  size = 16,
  className = "",
  alt = "",
  fallback = "/icons/filetypes/document.svg",
  invert,
}: SvgIconProps) {
  const [error, setError] = useState(false);
  const [loaded, setLoaded] = useState(false);

  const handleError = () => {
    if (!error && fallback && src !== fallback) {
      setError(true);
    }
  };

  const handleLoad = () => {
    setLoaded(true);
  };

  const iconSrc = error ? fallback : src;

  // Auto-detect if inversion is needed based on icon path
  // UI icons from Fluent are dark-on-light, so they need inversion for dark theme
  const shouldInvert = invert ?? src.includes("/icons/ui/");

  return (
    <img
      src={iconSrc}
      width={size}
      height={size}
      alt={alt}
      className={`inline-block shrink-0 transition-opacity duration-100 ${shouldInvert ? "invert" : ""} ${className} ${loaded ? "opacity-100" : "opacity-0"}`}
      onError={handleError}
      onLoad={handleLoad}
      loading="lazy"
      decoding="async"
    />
  );
});

export default SvgIcon;
