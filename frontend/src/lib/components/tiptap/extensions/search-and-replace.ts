import { type Editor as CoreEditor, Extension, type Range } from '@tiptap/core';
import type { Node as PMNode } from '@tiptap/pm/model';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';

export interface SearchAndReplaceStorage {
  searchTerm: string;
  replaceTerm: string;
  results: Range[];
  lastSearchTerm: string;
  selectedResult: number;
  lastSelectedResult: number;
  caseSensitive: boolean;
  lastCaseSensitiveState: boolean;
}

declare module '@tiptap/core' {
  interface Storage {
    searchAndReplace: SearchAndReplaceStorage;
  }

  interface Commands<ReturnType> {
    search: {
      /**
       * @description Set search term in extension.
       */
      setSearchTerm: (searchTerm: string) => ReturnType;
      /**
       * @description Set replace term in extension.
       */
      setReplaceTerm: (replaceTerm: string) => ReturnType;
      /**
       * @description Replace first instance of search result with given replace term.
       */
      replace: () => ReturnType;
      /**
       * @description Replace all instances of search result with given replace term.
       */
      replaceAll: () => ReturnType;
      /**
       * @description Select the next search result.
       */
      selectNextResult: () => ReturnType;
      /**
       * @description Select the previous search result.
       */
      selectPreviousResult: () => ReturnType;
      /**
       * @description Set case sensitivity in extension.
       */
      setCaseSensitive: (caseSensitive: boolean) => ReturnType;
    };
  }
}

interface TextNodeWithPosition {
  text: string;
  pos: number;
}

const getRegex = (
  searchString: string,
  disableRegex: boolean,
  caseSensitive: boolean
): RegExp => {
  const escapedString = disableRegex
    ? searchString.replace(/[-/\\^$*+?.()|[\]{}]/g, String.raw`\$&`)
    : searchString;
  return new RegExp(escapedString, caseSensitive ? 'gu' : 'gui');
};

interface ProcessedSearches {
  decorationsToReturn: DecorationSet;
  results: Range[];
}

const processSearches = (
  doc: PMNode,
  searchTerm: RegExp,
  selectedResultIndex: number,
  searchResultClass: string,
  selectedResultClass: string
): ProcessedSearches => {
  const decorations: Decoration[] = [];
  const results: Range[] = [];
  const textNodesWithPosition: TextNodeWithPosition[] = [];

  if (!searchTerm) {
    return { decorationsToReturn: DecorationSet.empty, results: [] };
  }

  doc.descendants((node, pos) => {
    if (node.isText) {
      textNodesWithPosition.push({ pos, text: node.text || '' });
    }
  });

  for (const { text, pos } of textNodesWithPosition) {
    const matches = [...text.matchAll(searchTerm)].filter(
      ([matchText]) => matchText.trim()
    );

    for (const match of matches) {
      if (match.index !== undefined) {
        results.push({
          from: pos + match.index,
          to: pos + match.index + match[0].length
        });
      }
    }
  }

  for (let i = 0; i < results.length; i += 1) {
    const result = results[i];
    if (!result) {continue;}
    const { from, to } = result;
    decorations.push(
      Decoration.inline(from, to, {
        class:
          selectedResultIndex === i ? selectedResultClass : searchResultClass
      })
    );
  }

  return {
    decorationsToReturn: DecorationSet.create(doc, decorations),
    results
  };
};

const replace = (
  replaceTerm: string,
  results: Range[],
  { state, dispatch }: any
) => {
  const [firstResult] = results;

  if (!firstResult) {
    return;
  }

  const { from, to } = firstResult;

  if (dispatch) {
    dispatch(state.tr.insertText(replaceTerm, from, to));
  }
};

const rebaseNextResult = (
  replaceTerm: string,
  index: number,
  lastOffset: number,
  results: Range[]
): [number, Range[]] | undefined => {
  const nextIndex = index + 1;

  if (!results[nextIndex]) {
    return undefined;
  }

  const currentResult = results[index];
  if (!currentResult) {
    return undefined;
  }

  const { from: currentFrom, to: currentTo } = currentResult;

  const offset = currentTo - currentFrom - replaceTerm.length + lastOffset;

  const { from, to } = results[nextIndex];

  results[nextIndex] = {
    from: from - offset,
    to: to - offset
  };

  return [offset, results];
};

const replaceAll = (
  replaceTerm: string,
  results: Range[],
  { tr, dispatch }: { tr: any; dispatch: any }
) => {
  if (!results.length) {
    return;
  }

  let offset = 0;

  for (let i = 0; i < results.length; i += 1) {
    const result = results[i];
    if (!result) {continue;}
    const { from, to } = result;
    tr.insertText(replaceTerm, from, to);
    const rebaseResponse = rebaseNextResult(replaceTerm, i, offset, results);

    if (rebaseResponse) {
      const [nextOffset] = rebaseResponse;
      offset = nextOffset;
    }
  }

  dispatch(tr);
};

const selectNext = (editor: CoreEditor) => {
  const { results } = editor.storage.searchAndReplace;

  if (!results.length) {
    return;
  }

  const { selectedResult } = editor.storage.searchAndReplace;

  if (selectedResult >= results.length - 1) {
    editor.storage.searchAndReplace.selectedResult = 0;
  } else {
    editor.storage.searchAndReplace.selectedResult += 1;
  }

  const result = results[editor.storage.searchAndReplace.selectedResult];
  if (!result) {return;}

  const { from } = result;

  const {view} = editor;

  if (view) {
    view
      .domAtPos(from)
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      .node.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }
};

const selectPrevious = (editor: CoreEditor) => {
  const { results } = editor.storage.searchAndReplace;

  if (!results.length) {
    return;
  }

  const { selectedResult } = editor.storage.searchAndReplace;

  if (selectedResult <= 0) {
    editor.storage.searchAndReplace.selectedResult = results.length - 1;
  } else {
    editor.storage.searchAndReplace.selectedResult -= 1;
  }

  const { from } = results[editor.storage.searchAndReplace.selectedResult];

  const {view} = editor;

  if (view) {
    view
      .domAtPos(from)
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      .node.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }
};

export const searchAndReplacePluginKey = new PluginKey(
  'searchAndReplacePlugin'
);

export interface SearchAndReplaceOptions {
  searchResultClass: string;
  selectedResultClass: string;
  disableRegex: boolean;
}

export const SearchAndReplace = Extension.create<
  SearchAndReplaceOptions,
  SearchAndReplaceStorage
>({
  addCommands() {
    return {
      replace:
        () =>
        ({ editor, state, dispatch }) => {
          const { replaceTerm, results } = editor.storage.searchAndReplace;

          replace(replaceTerm, results, { dispatch, state });

          return false;
        },
      replaceAll:
        () =>
        ({ editor, tr, dispatch }) => {
          const { replaceTerm, results } = editor.storage.searchAndReplace;

          replaceAll(replaceTerm, results, { dispatch, tr });

          return false;
        },
      selectNextResult:
        () =>
        ({ editor }) => {
          selectNext(editor);

          return false;
        },
      selectPreviousResult:
        () =>
        ({ editor }) => {
          selectPrevious(editor);

          return false;
        },
      setCaseSensitive:
        (caseSensitive: boolean) =>
        ({ editor }) => {
          editor.storage.searchAndReplace.caseSensitive = caseSensitive;

          return false;
        },
      setReplaceTerm:
        (replaceTerm: string) =>
        ({ editor }) => {
          editor.storage.searchAndReplace.replaceTerm = replaceTerm;

          return false;
        },
      setSearchTerm:
        (searchTerm: string) =>
        ({ editor }) => {
          editor.storage.searchAndReplace.searchTerm = searchTerm;

          return false;
        }
    };
  },

  addOptions() {
    return {
      disableRegex: true,
      searchResultClass: ' bg-yellow-200',
      selectedResultClass: 'bg-yellow-500'
    };
  },

  addProseMirrorPlugins() {
    const {editor} = this;
    const { searchResultClass, selectedResultClass, disableRegex } =
      this.options;

    const setLastSearchTerm = (t: string) => {
      editor.storage.searchAndReplace.lastSearchTerm = t;
    };

    const setLastSelectedResult = (r: number) => {
      editor.storage.searchAndReplace.lastSelectedResult = r;
    };

    const setLastCaseSensitiveState = (s: boolean) => {
      editor.storage.searchAndReplace.lastCaseSensitiveState = s;
    };

    return [
      new Plugin({
        key: searchAndReplacePluginKey,
        props: {
          decorations(state) {
            return this.getState(state);
          }
        },
        state: {
          apply({ doc, docChanged }, oldState) {
            const {
              searchTerm,
              selectedResult,
              lastSearchTerm,
              lastSelectedResult,
              caseSensitive,
              lastCaseSensitiveState
            } = editor.storage.searchAndReplace;

            if (
              !docChanged &&
              lastSearchTerm === searchTerm &&
              selectedResult === lastSelectedResult &&
              lastCaseSensitiveState === caseSensitive
            ) {
              return oldState;
            }

            setLastSearchTerm(searchTerm);
            setLastSelectedResult(selectedResult);
            setLastCaseSensitiveState(caseSensitive);

            if (!searchTerm) {
              editor.storage.searchAndReplace.selectedResult = 0;
              editor.storage.searchAndReplace.results = [];
              return DecorationSet.empty;
            }

            const { decorationsToReturn, results } = processSearches(
              doc,
              getRegex(searchTerm, disableRegex, caseSensitive),
              selectedResult,
              searchResultClass,
              selectedResultClass
            );

            editor.storage.searchAndReplace.results = results;

            if (selectedResult > results.length) {
              editor.storage.searchAndReplace.selectedResult =
                results.length > 0 ? results.length : 0;
            }

            return decorationsToReturn;
          },
          init: () => DecorationSet.empty
        }
      })
    ];
  },

  addStorage() {
    return {
      caseSensitive: false,
      lastCaseSensitiveState: false,
      lastSearchTerm: '',
      lastSelectedResult: 0,
      replaceTerm: '',
      results: [],
      searchTerm: '',
      selectedResult: 0
    };
  },

  name: 'searchAndReplace'
});

export default SearchAndReplace;
