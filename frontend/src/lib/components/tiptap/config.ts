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
