import React from "react";
import ReactPlayer from "react-player";
import { VideoStyleProps, makeStyles } from "./Video.style";

export type VideoProps = {
  url?: string;
  playing?: boolean;
  poster?: string;
  controls?: boolean;
  volume?: number;
  loop?: boolean;
  autoPlay?: boolean;
  muted?: boolean;
  className?: string;
  onReady?(): void;
  onStart?(): void;
  onPlay?(): void;
  onPause?(): void;
  onBuffer?(): void;
  onEnded?(): void;
  onError?(error: any): void;
  onDuration?(duration: number): void;
  onProgress?(state: { played: number; loaded: number }): void;
} & VideoStyleProps;

export default function Video({
  url,
  poster,
  playing,
  onPause,
  autoPlay,
  loop = false,
  muted = true,
  onStart,
  ratio,
  onReady,
  onPlay,
  onBuffer,
  onError,
  onEnded,
  onDuration,
  onProgress,
  className,
  volume = 0.7,
  controls = true,
  ...styleProps
}: VideoProps) {
  let { playerStyles, containerStyles } = makeStyles(styleProps);
  return (
    <div css={containerStyles}>
      <ReactPlayer
        css={playerStyles}
        width={styleProps.responsive ? "100%" : styleProps.width}
        height={styleProps.responsive ? "100%" : styleProps.height}
        url={url}
        autoPlay={autoPlay}
        light={poster || true}
        className={className}
        playing={playing}
        loop={loop}
        controls={controls}
        onStart={onStart}
        onPlay={onPlay}
        onBuffer={onBuffer}
        onReady={onReady}
        onEnded={onEnded}
        onDuration={onDuration}
        onProgress={onProgress}
      />
    </div>
  );
}
