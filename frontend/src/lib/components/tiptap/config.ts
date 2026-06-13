import TextAlign from '@tiptap/extension-text-align';
import { Color, TextStyle } from '@tiptap/extension-text-style';
import { Highlight } from '@tiptap/extension-highlight';
import Typography from '@tiptap/extension-typography';
import StarterKit from '@tiptap/starter-kit';
import type { Extensions } from '@tiptap/core';
import SearchAndReplace from './extensions/search-and-replace';
import CharacterCount from '@tiptap/extension-character-count';

export const extensions = [
  StarterKit.configure({
    bulletList: {
      HTMLAttributes: {
        class: 'list-disc'
      }
    },
    heading: {
      levels: [1, 2, 3, 4]
    },
    orderedList: {
      HTMLAttributes: {
        class: 'list-decimal'
      }
    },
    undoRedo: false
  }),
  TextAlign.configure({
    types: ['heading', 'paragraph']
  }),
  TextStyle,
  Color,
  Highlight.configure({
    multicolor: true
  }),
  SearchAndReplace,
  Typography,
  CharacterCount.configure({
    autoTrim: true,
    limit: 50_000,
    mode: 'nodeSize'
  })
] satisfies Extensions;

const AVATAR_COLORS = [
  '#958DF1',
  '#F98181',
  '#FBBC88',
  '#FAF594',
  '#70CFF8',
  '#94FADB',
  '#B9F18D',
  '#FF85A2'
];

export const getRandomColor = () =>
  AVATAR_COLORS[Math.floor(Math.random() * AVATAR_COLORS.length)];
