import type { BoxProps } from '@mui/material/Box';
import type { IKanbanComment } from 'src/types/kanban';

import Box from '@mui/material/Box';
import Avatar from '@mui/material/Avatar';
import Typography from '@mui/material/Typography';

import { fToNow } from 'src/utils/format-time';

import { Image } from 'src/components/image';
import { Lightbox, useLightbox } from 'src/components/lightbox';

// ----------------------------------------------------------------------

type Props = BoxProps & {
  comments: IKanbanComment[];
};

export function KanbanDetailsCommentList({ comments, sx, ...other }: Props) {
  const slides = comments
    .filter((comment) => comment.messageType === 'image')
    .map((slide) => ({ src: slide.message }));

  const lightbox = useLightbox(slides);

  return (
    <>
      <Box
        component="ul"
        sx={[
          {
            gap: 3,
            display: 'flex',
            flexDirection: 'column',
          },
          ...(Array.isArray(sx) ? sx : [sx]),
        ]}
        {...other}
      >
        {comments.map((comment) => (
          <Box component="li" key={comment.id} sx={{ gap: 2, display: 'flex' }}>
            <Avatar src={comment.avatarUrl} />

            <Box
              sx={{
                display: 'flex',
                flex: '1 1 auto',
                flexDirection: 'column',
                gap: comment.messageType === 'image' ? 1 : 0.5,
              }}
            >
              <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <Typography variant="subtitle2"> {comment.name}</Typography>
                <Typography variant="caption" sx={{ color: 'text.disabled' }}>
                  {fToNow(comment.createdAt)}
                </Typography>
              </Box>

              {comment.messageType === 'image' ? (
                <Image
                  alt={comment.message}
                  src={comment.message}
                  onClick={() => lightbox.onOpen(comment.message)}
                  sx={(theme) => ({
                    borderRadius: 1.5,
                    cursor: 'pointer',
                    transition: theme.transitions.create(['opacity']),
                    '&:hover': { opacity: 0.8 },
                  })}
                />
              ) : (
                <Typography variant="body2">{comment.message}</Typography>
              )}
            </Box>
          </Box>
        ))}
      </Box>

      <Lightbox
        index={lightbox.selected}
        slides={slides}
        open={lightbox.open}
        close={lightbox.onClose}
      />
    </>
  );
}
