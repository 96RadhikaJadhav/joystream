import { css } from "@emotion/core"
import { typography, colors } from "./../../theme"

export type VideoPreviewStyleProps = {
  poster?: string
}

export let makeStyles = ({ poster }: VideoPreviewStyleProps) => {
  return {
    container: css``,
    containerLink: css`
      text-decoration: none;
    `,
    coverContainer: css`
      width: 100%;
      background-color: black;
    `,
    cover: css`
      display: block;
      width: 100%;
      height: auto;
    `,
    infoContainer: css`
      display: grid;
      grid-template: auto / 45px auto;
      margin: 10px 0 0;
    `,
    avatar: css`
      grid-column: 1 / 1;
    `,
    textContainer: css`
      grid-column: 2 / 2;
    `,
    title: css`
      margin: 0;
      font-weight: ${typography.weights.bold};
      text-transform: capitalize;
      color: ${colors.black.base};
      font-size: ${typography.sizes.small};
    `,
    channel: css`
      text-decoration: none;
      & > h3 {
        margin: 5px 0 0;
        font-size: ${typography.sizes.xsmall};
        color: ${colors.grey.darker};
      }
    `
  }
}
