import sys
from PySide2 import QtWidgets, QtGui, QtCore


class ThingyListModel(QtCore.QAbstractListModel):
    # TODO:
    def __init__(self, parent=None):
        super().__init__(parent)

        self._underlying = [
            "Thing 1",
            "Thing 2",
            "Thing 3"
        ]

    def rowCount(self, parent=QtCore.QModelIndex()):
        return len(self._underlying)

    def setData(self, index, value, role):
        if index.isValid() and role == QtCore.Qt.EditRole:
            self._underlying[index.row()] = value

            self.dataChanged.emit(index, index, [role])

    def data(self, index, role=QtCore.Qt.DisplayRole):
        if role == QtCore.Qt.DisplayRole:
            item = self._underlying[index.row()]
            print("Asked for:", index, item)
            return item

        return None


class MainWindow(QtWidgets.QWidget):
    def __init__(self):
        super().__init__()

        self._model = ThingyListModel(self)

        self.listview = QtWidgets.QListView()
        self.listview.setModel(self._model)

        # This is just to demonstrate that the vie does indeed change
        # if we change the model
        def setdatastuff():
            self._model.setData(self._model.createIndex(2, 0, 0), "Thing X", QtCore.Qt.EditRole)

        self._timer = QtCore.QTimer(self)
        self._timer.timeout.connect(setdatastuff)
        self._timer.start(1000)
        # End of the timer goofings

        self._layout = QtWidgets.QVBoxLayout()
        self._layout.addWidget(self.listview)

        self.setLayout(self._layout)


def main():
    app = QtWidgets.QApplication(sys.argv)

    main_window = MainWindow()
    main_window.show()

    sys.exit(app.exec_())

