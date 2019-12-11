
/** This file is generated based on JSON schema. Do not modify. */

import * as Yup from 'yup';

export const ContentLicenseValidationSchema = Yup.object().shape({
  license: Yup.string()
    .required('This field is required')
    .max(200, 'Text is too long. Maximum length is 200 chars.')
});

export type ContentLicenseType = {
  license: string
};

export const ContentLicenseClass = {
  license: {
    "name": "License",
    "description": "The license of which the content is originally published under.",
    "type": "Text",
    "required": true,
    "maxTextLength": 200
  }
};
