'use client';

import type { IconButtonProps } from '@mui/material/IconButton';
import type { IconifyName } from 'src/components/iconify';
import type { ContactMethod } from 'src/types/system-setting';

import { useState } from 'react';
import { m } from 'framer-motion';
import { usePopover } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Avatar from '@mui/material/Avatar';
import Dialog from '@mui/material/Dialog';
import MenuItem from '@mui/material/MenuItem';
import MenuList from '@mui/material/MenuList';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import DialogTitle from '@mui/material/DialogTitle';
import ListItemText from '@mui/material/ListItemText';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales';
import { useSiteInfo } from 'src/actions/system-settings';

import { Scrollbar } from 'src/components/scrollbar';
import { CustomPopover } from 'src/components/custom-popover';
import { Iconify, allIconNames } from 'src/components/iconify';
import { varTap, varHover, transitionTap } from 'src/components/animate';

export type ContactsPopoverProps = IconButtonProps;

export function ContactsPopover({ sx, ...other }: ContactsPopoverProps) {
  const { t } = useTranslate('common');
  const { data } = useSiteInfo();
  const { open, anchorEl, onClose, onOpen } = usePopover();
  const [selected, setSelected] = useState<ContactMethod | null>(null);
  const contacts = data?.contact_methods ?? [];

  if (!contacts.length) {
    return null;
  }

  return (
    <>
      <IconButton
        component={m.button}
        whileTap={varTap(0.96)}
        whileHover={varHover(1.04)}
        transition={transitionTap()}
        aria-label="Contacts"
        onClick={onOpen}
        sx={[
          (theme) => ({ ...(open && { bgcolor: theme.vars.palette.action.selected }) }),
          ...(Array.isArray(sx) ? sx : [sx]),
        ]}
        {...other}
      >
        <Iconify icon="solar:users-group-rounded-bold-duotone" width={24} />
      </IconButton>

      <ContactList
        open={open}
        anchorEl={anchorEl}
        contacts={contacts}
        onClose={onClose}
        onSelect={setSelected}
        t={t}
      />
      <ContactDetailDialog contact={selected} onClose={() => setSelected(null)} t={t} />
    </>
  );
}

function ContactList({
  open,
  anchorEl,
  contacts,
  onClose,
  onSelect,
  t,
}: {
  open: boolean;
  anchorEl: HTMLElement | null;
  contacts: ContactMethod[];
  onClose: () => void;
  onSelect: (contact: ContactMethod) => void;
  t: (key: string) => string;
}) {
  return (
    <CustomPopover open={open} anchorEl={anchorEl} onClose={onClose}>
      <Typography variant="h6" sx={{ p: 1.5 }}>
        {t('contacts.title')} <span>({contacts.length})</span>
      </Typography>

      <Scrollbar sx={{ height: 320, width: 320 }}>
        <MenuList>
          {contacts.map((contact) => (
            <ContactListItem
              key={contact.id}
              contact={contact}
              t={t}
              onClick={() => {
                onSelect(contact);
                onClose();
              }}
            />
          ))}
        </MenuList>
      </Scrollbar>
    </CustomPopover>
  );
}

function ContactListItem({
  contact,
  onClick,
  t,
}: {
  contact: ContactMethod;
  onClick: () => void;
  t: (key: string) => string;
}) {
  const title = contactTitle(contact, t);

  return (
    <MenuItem onClick={onClick} sx={{ p: 1 }}>
      <Avatar sx={{ bgcolor: 'background.neutral', color: 'text.primary' }}>
        <Iconify icon={contactIcon(contact.icon)} width={24} />
      </Avatar>

      <ListItemText
        primary={title}
        secondary={contact.qr_code ? t('contacts.viewQrCode') : contact.value}
        slotProps={{
          primary: { noWrap: true },
          secondary: {
            noWrap: true,
            sx: { typography: 'caption', color: 'text.disabled' },
          },
        }}
      />
    </MenuItem>
  );
}

function ContactDetailDialog({
  contact,
  onClose,
  t,
}: {
  contact: ContactMethod | null;
  onClose: () => void;
  t: (key: string) => string;
}) {
  if (!contact) {
    return null;
  }

  return (
    <Dialog open onClose={onClose} fullWidth maxWidth="xs">
      <DialogTitle sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}>
        <Avatar sx={{ bgcolor: 'background.neutral', color: 'text.primary' }}>
          <Iconify icon={contactIcon(contact.icon)} width={24} />
        </Avatar>
        {contactTitle(contact, t)}
      </DialogTitle>

      <DialogContent sx={{ pt: 1, pb: 3 }}>
        <Typography variant="caption" sx={{ color: 'text.disabled' }}>
          {t('contacts.value')}
        </Typography>
        <Typography variant="body2" sx={{ mt: 0.5, wordBreak: 'break-word' }}>
          {contact.value}
        </Typography>

        {contact.qr_code && (
          <Box sx={{ mt: 3 }}>
            <Typography variant="caption" sx={{ color: 'text.disabled' }}>
              {t('contacts.qrCode')}
            </Typography>
            <Box
              component="img"
              alt={contactTitle(contact, t)}
              src={contact.qr_code}
              sx={{
                mt: 1,
                width: 1,
                mx: 'auto',
                maxWidth: 240,
                display: 'block',
                borderRadius: 1,
                border: (theme) => `1px solid ${theme.vars.palette.divider}`,
              }}
            />
          </Box>
        )}
      </DialogContent>
    </Dialog>
  );
}

function contactTitle(contact: ContactMethod, t: (key: string) => string) {
  if (contact.type === 'custom') {
    return contact.custom_type;
  }
  return t(`contacts.types.${contact.type}`);
}

function contactIcon(icon: string): IconifyName {
  if (allIconNames.includes(icon as IconifyName)) {
    return icon as IconifyName;
  }
  return 'solar:chat-round-dots-bold';
}
