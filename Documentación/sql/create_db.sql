CRATE DATABASE WifiViewer;

CREATE USER WifiVisualizerAdmin IDENTIFIED BY 'AdminSosigWifiVisualizer%$';
GRANT ALL PRIVILEGES ON *.* TO WifiVisualizerAdmin;

-- DROP USER WifiVisualizerUser;
CREATE USER WifiVisualizerUser IDENTIFIED BY '';
GRANT SELECT, INSERT, UPDATE ON WifiViewer.* TO 'WifiVisualizerUser';

