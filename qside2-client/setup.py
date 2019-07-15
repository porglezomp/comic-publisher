import setuptools

setuptools.setup(
    name="comic_publisher",
    version="0.1.0",
    author="fennecs",

    entry_points={
        "console_scripts": [
            "comic-publisher-pyside2=comic_publisher.main:main"
        ]
    }
)


