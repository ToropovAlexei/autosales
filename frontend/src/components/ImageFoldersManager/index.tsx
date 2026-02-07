import {
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  Typography,
} from "@mui/material";
import classes from "./styles.module.css";
import { IMAGE_FOLDERS } from "@/constants";
import { useState } from "react";
import { ImageFolder } from "../ImageFolder";
import { ImageResponse } from "@/types";

interface IProps {
  initialFolder?: "other" | "product" | "category" | "fulfillment";
  onSelect?: (image: ImageResponse) => void;
}

export const ImageFoldersManager = ({ initialFolder, onSelect }: IProps) => {
  const [selectedFolder, setSelectedFolder] = useState(
    initialFolder || IMAGE_FOLDERS[0].id,
  );

  return (
    <div className={classes.container}>
      <div className={classes.sidebar}>
        <Typography variant="h6" gutterBottom>
          Папки
        </Typography>
        <List component="nav">
          {IMAGE_FOLDERS.map((folder) => (
            <ListItem key={folder.id} disablePadding>
              <ListItemButton
                selected={selectedFolder === folder.id}
                onClick={() =>
                  setSelectedFolder(
                    folder.id as
                      | "other"
                      | "product"
                      | "category"
                      | "fulfillment",
                  )
                }
              >
                <ListItemText primary={folder.name} />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
      </div>
      <div className={classes.mainContent}>
        <ImageFolder folder={selectedFolder} onSelect={onSelect} />
      </div>
    </div>
  );
};
