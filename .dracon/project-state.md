# Project State

## Current Focus
Add ContextMenuAction enum defining all actions available in the file/folder context menu.

## Completed
- [x] Added enum variant `Open` to open the selected file or directory
- [x] Added enum variant `OpenNewTab` to open in a new tab
- [x] Added enum variant `OpenWith` to open with a specific application
- [x] Added enum variant `Edit` to edit the selected item
- [x] Added enum variant `Run` to execute the selected file
- [x] Added enum variant `RunTerminal` to run in a new terminal window
- [x] Added enum variant `ExtractHere` to extract an archive in the current directory
- [x] Added enum variant `NewFolder` to create a new folder
- [x] Added enum variant `NewFile` to create a new file
- [x] Added enum variant `Cut` to cut selected items to clipboard
- [x] Added enum variant `Copy` to copy selected items to clipboard
- [x] Added enum variant `CopyPath` to copy the full path of the item
- [x] Added enum variant `CopyName` to copy only the item's name
- [x] Added enum variant `Paste` to paste from clipboard
- [x] Added enum variant `Rename` to rename the selected item
- [x] Added enum variant `Duplicate` to duplicate the selected item
- [x] Added enum variant `Compress` to compress selected items into an archive
- [x] Added enum variant `Delete` to delete selected items
- [x] Added enum variant `TerminalWindow` to open a new terminal window
- [x] Added enum variant `TerminalTab` to open a new terminal tab
- [x] Added enum variant `SetColor(Option<u8>)` to set terminal background color
- [x] Added enum variant `Properties` to show item properties
- [x] Added enum variant `GitStatus` to display Git status for the item
- [x] Added enum variants `AddToFavorites` and `RemoveFromFavorites` to manage favorites
- [x] Added enum variant `Refresh` to refresh the current view
- [x] Added enum variant `SelectAll` to select all items
- [x] Added enum variant `ToggleHidden` to toggle visibility of hidden files
- [x] Added enum variants `ConnectRemote`, `DeleteRemote`, `Mount`, and `Unmount` for remote filesystem operations
- [x] Added enum variant `SetWallpaper` to set the terminal wallpaper
- [x] Added enum variant `GitInit` to initialize a Git repository
- [x] Added enum variant `SystemMonitor` to open the system monitor
- [x] Added enum variant `Drag` to initiate a drag operation
- [x] Added enum variant `SortBy(FileColumn)` to sort entries by a specific column
- [x] Added enum variant `Separator` to provide a visual separator between menu sections
- [x] Updated `Cargo.lock` dependency lock file (binary unchanged)
