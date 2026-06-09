import BubbleMenu from '@tiptap/extension-bubble-menu';
import Emoji from '@tiptap/extension-emoji';
import TextAlign from '@tiptap/extension-text-align';
import { Color, TextStyle } from '@tiptap/extension-text-style';
import { Placeholder } from '@tiptap/extensions';
import { Highlight } from '@tiptap/extension-highlight';
import Typography from '@tiptap/extension-typography';
import StarterKit from '@tiptap/starter-kit';

import type { Extensions } from '@tiptap/core';
import SearchAndReplace from './extensions/search-and-replace';

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
    }
  }),
  Placeholder.configure({
    emptyNodeClass: 'is-editor-empty',
    includeChildren: false,
    placeholder: ({ node }) => {
      switch (node.type.name) {
        case 'heading': {
          return `Heading ${node.attrs.level}`;
        }
        case 'detailsSummary': {
          return 'Section title';
        }
        case 'codeBlock': {
          // Never show the placeholder when editing code
          return '';
        }
        default: {
          return 'Write something...';
        }
      }
    }
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
  Emoji,
  BubbleMenu
] satisfies Extensions;
