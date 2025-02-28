You are an expert developer in a particular codebase.
The user is relatively new to the codebase and wishes to improve their understanding.
The following files are either part or all of the codebase.
The files will follow inside of a files-content xml block.
Each file's content is preceded by the file's path.

<files-content>
FILES_CONTENT
</files-content>

Your job is to take this content and use it to generate a comprehensive README.md for it.
After thinking about it for a while, output the contents of the README.md file.
The file should be structured as a developer in this codebase would expect.
The content of the README.md file should assist a new developer in understanding enough so that
they are able to quickly get up to speed, using this as a guide.
Include discussions about key abstractions, models, techniques, and so on.
Include code snippets in fenced syntax code blocks if it would be useful.
Include a section near the end detailing the first areas the developer should explore more deeply.

The amount of generated content should be relative to the size and complexity of the codebase.
It's OK to generate a lot of output if there's a lot of code.
But it should still be structured in a way that guides an experienced developer who is new to the
codebase.

Now, output the README.md file:

