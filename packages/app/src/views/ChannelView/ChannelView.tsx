import React from "react";
import { GeneralSection, Avatar, Button, VideoPreview } from "components";
import { ChannelHeader } from "../../components/ChannelHeader";

type ChannelProps = {
  name: string;
  isPublic?: boolean;
  isVerified?: boolean;
  description?: string;
  banner?: string;
  videos?: any[];
  img: string;
};

export default function Channel({
  name,
  isPublic = true,
  isVerified = false,
  description,
  videos,
  img,
}: ChannelProps) {
  return (
    <>
      <ChannelHeader
        name={name}
        isPublic={isPublic}
        isVerified={isVerified}
        description={description}
        img={img}
      />
      <GeneralSection auto title="Videos">
        <div className="video-gallery">
          {videos.map((video, key) => (
            <VideoPreview
              url="#"
              channelUrl="#"
              key={`title-${key}`}
              title={video.title}
              poster={video.poster}
              channel={name}
              channelImg={img}
              showChannel
            />
          ))}
        </div>
      </GeneralSection>
    </>
  );
}
