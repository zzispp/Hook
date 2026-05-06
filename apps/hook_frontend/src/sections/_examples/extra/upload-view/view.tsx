'use client';

import type { PreviewOrientation } from 'src/components/upload';
import type { FileThumbnailProps } from 'src/components/file-thumbnail';

import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';

import { fData } from 'src/utils/format-number';

import { Iconify } from 'src/components/iconify';
import { Upload, UploadBox, UploadAvatar } from 'src/components/upload';

import { ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

export function UploadView() {
  const [files, setFiles] = useState<(File | string)[]>([]);
  const [file, setFile] = useState<File | string | null>(null);
  const [avatarUrl, setAvatarUrl] = useState<File | string | null>(null);

  const [preview, setPreview] = useState<{
    orientation: PreviewOrientation;
    showImage: FileThumbnailProps['showImage'];
  }>({
    orientation: 'horizontal',
    showImage: true,
  });

  const handleDropSingleFile = useCallback((acceptedFiles: File[]) => {
    setFile(acceptedFiles[0]);
  }, []);

  const handleDropAvatar = useCallback((acceptedFiles: File[]) => {
    setAvatarUrl(acceptedFiles[0]);
  }, []);

  const handleDropMultiFile = useCallback((acceptedFiles: File[]) => {
    setFiles((prevFiles) => [...prevFiles, ...acceptedFiles]);
  }, []);

  const handleRemoveFile = useCallback((inputFile: File | string) => {
    setFiles((prevFiles) => prevFiles.filter((fileFiltered) => fileFiltered !== inputFile));
  }, []);

  const handleRemoveAllFiles = useCallback(() => {
    setFiles([]);
  }, []);

  const DEMO_COMPONENTS = [
    {
      name: 'Upload single file',
      component: (
        <Upload value={file} onDrop={handleDropSingleFile} onDelete={() => setFile(null)} />
      ),
    },
    {
      name: 'Upload multi file',
      component: (
        <>
          <Box sx={{ mb: 3, display: 'flex', justifyContent: 'flex-end' }}>
            <FormControlLabel
              label="Horizontal layout"
              control={
                <Switch
                  checked={preview.orientation === 'horizontal'}
                  onClick={() =>
                    setPreview((prev) => ({
                      ...prev,
                      orientation: prev.orientation === 'horizontal' ? 'vertical' : 'horizontal',
                    }))
                  }
                  slotProps={{ input: { id: 'layout-switch' } }}
                />
              }
            />

            <FormControlLabel
              label="Show image"
              control={
                <Switch
                  checked={preview.showImage}
                  onClick={() =>
                    setPreview((prev) => ({
                      ...prev,
                      showImage: !prev.showImage,
                    }))
                  }
                  slotProps={{ input: { id: 'view-switch' } }}
                />
              }
            />
          </Box>

          <Upload
            multiple
            value={files}
            onDrop={handleDropMultiFile}
            onRemove={handleRemoveFile}
            onRemoveAll={handleRemoveAllFiles}
            onUpload={() => console.info('ON UPLOAD')}
            previewOrientation={preview.orientation}
            slotProps={{
              multiPreview: {
                thumbnail: { showImage: preview.showImage },
              },
            }}
          />
        </>
      ),
    },
    {
      name: 'Upload avatar',
      component: (
        <UploadAvatar
          value={avatarUrl}
          onDrop={handleDropAvatar}
          validator={(fileData) => {
            if (fileData.size > 1000000) {
              return {
                code: 'file-too-large',
                message: `File is larger than ${fData(1000000)}`,
              };
            }
            return null;
          }}
          helperText={
            <Typography
              variant="caption"
              sx={{
                mt: 3,
                mx: 'auto',
                display: 'block',
                textAlign: 'center',
                color: 'text.disabled',
              }}
            >
              Allowed *.jpeg, *.jpg, *.png, *.gif
              <br /> max size of {fData(3145728)}
            </Typography>
          }
        />
      ),
    },
    {
      name: 'Upload box',
      component: (
        <Box sx={{ display: 'flex', gap: 2 }}>
          <UploadBox />
          <UploadBox
            placeholder={
              <Stack spacing={0.5} sx={{ alignItems: 'center' }}>
                <Iconify icon="eva:cloud-upload-fill" width={40} />
                <Typography variant="body2">Upload file</Typography>
              </Stack>
            }
            sx={{
              mb: 3,
              py: 2.5,
              flexGrow: 1,
              height: 'auto',
            }}
          />
        </Box>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Upload',
        moreLinks: ['https://react-dropzone.js.org/#section-basic-example'],
      }}
    />
  );
}
