import * as z from 'zod';
import { isValidPhoneNumber } from 'react-phone-number-input/input';

import { fIsAfter } from 'src/utils/format-time';

import { schemaUtils } from 'src/components/hook-form';

// ----------------------------------------------------------------------

export type FieldsSchemaType = z.infer<typeof FieldsSchema>;

export const FieldsSchema = z
  .object({
    fullName: z
      .string()
      .min(1, { error: 'Full name is required!' })
      .min(6, { error: 'Mininum 6 characters!' })
      .max(32, { error: 'Maximum 32 characters!' }),
    email: schemaUtils.email(),
    age: schemaUtils.nullableInput(
      z.coerce
        .number()
        .int()
        .min(1, { error: 'Age is required!' })
        .min(18, { error: 'Age must be between 18 and 80' })
        .max(80, { error: 'Age must be between 18 and 80' }),
      { error: 'Age is required!' }
    ),
    price: schemaUtils.nullableInput(
      z.coerce.number().min(1, { error: 'Price is required!' }).optional(),
      { error: 'Price is required!' }
    ),
    quantity: schemaUtils.nullableInput(
      z.coerce
        .number()
        .min(1, { error: 'Quantity is required!' })
        .max(99, { error: 'Quantity must be between 1 and 99' }),
      { error: 'Quantity is required!' }
    ),
    // phone
    phoneNumber: schemaUtils.phoneNumber({ isValid: isValidPhoneNumber }),
    // code
    code: z
      .string()
      .min(1, { error: 'Code is required!' })
      .min(6, { error: 'Code must be at least 6 characters!' }),
    // date
    startDate: schemaUtils.date({ error: { required: 'Start date is required!' } }),
    endDate: schemaUtils.date({ error: { required: 'End date is required!' } }),
    // password
    password: z
      .string()
      .min(1, { error: 'Password is required!' })
      .min(6, { error: 'Password is too short!' }),
    confirmPassword: z.string().min(1, { error: 'Confirm password is required!' }),
    // autocomplete
    singleAutocomplete: schemaUtils.nullableInput(z.custom<{ value: string; label: string }>(), {
      error: 'Autocomplete is required!',
    }),
    multiAutocomplete: z.array(z.object({ value: z.string(), label: z.string() })).min(2, {
      error: 'Must have at least 2 items!',
    }),
    // country
    singleCountry: z.string().min(1, { error: 'Single country is required!' }),
    multiCountry: z.string().array().min(2, { error: 'Must have at least 2 items!' }),
    // select
    singleSelect: z.string().min(1, { error: 'Single select is required!' }),
    multiSelect: z.string().array().min(2, { error: 'Must have at least 2 items!' }),
  })
  .refine((val) => val.password === val.confirmPassword, {
    error: 'Passwords do not match!',
    path: ['confirmPassword'],
  })
  .refine((val) => !fIsAfter(val.startDate, val.endDate), {
    error: 'End date cannot be earlier than start date!',
    path: ['endDate'],
  });

// ----------------------------------------------------------------------

export type ControlsSchemaType = z.infer<typeof ControlsSchema>;

export const ControlsSchema = z.object({
  // rating
  rating: z.number().min(1, { error: 'Rating is required!' }),
  // radio
  radioGroup: z.string().min(1, { error: 'Choose at least one option!' }),
  // checkbox
  checkbox: schemaUtils.boolean({ error: 'Checkbox is required!' }),
  multiCheckbox: z.string().array().min(1, { error: 'Choose at least one option!' }),
  // switch
  switch: schemaUtils.boolean({ error: 'Switch is required!' }),
  multiSwitch: z.string().array().min(1, { error: 'Choose at least one option!' }),
  // slider
  slider: z.number().min(10, { error: 'Mininum value is >= 10' }),
  sliderRange: schemaUtils.sliderRange({
    min: 20,
    max: 80,
  }),
});

// ----------------------------------------------------------------------

export type OtherSchemaType = z.infer<typeof OtherSchema>;

export const OtherSchema = z.object({
  editor: schemaUtils
    .editor()
    .min(100, { error: 'Content must be at least 100 characters' })
    .max(500, { error: 'Content must be less than 500 characters' }),
  singleUpload: schemaUtils.file({ error: 'Single upload is required!' }),
  multiUpload: schemaUtils.files({ error: 'Multi upload is required!' }).min(2, {
    error: 'Must have at least 2 items!',
  }),
});
